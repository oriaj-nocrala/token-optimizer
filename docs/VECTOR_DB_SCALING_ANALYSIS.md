/*! Vector Compression for Scalable Storage
 * Reduces memory usage by 75-97% while maintaining search quality
 */

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 8-bit quantized vector (75% memory reduction)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuantizedVector {
    /// Quantized values (0-255)
    pub values: Vec<u8>,
    /// Scale factor for reconstruction
    pub scale: f32,
    /// Offset for reconstruction
    pub offset: f32,
}

impl QuantizedVector {
    /// Quantize a full-precision vector to 8-bit
    pub fn from_f32(vector: &[f32]) -> Self {
        let min_val = vector.iter().copied().fold(f32::INFINITY, f32::min);
        let max_val = vector.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        
        let scale = (max_val - min_val) / 255.0;
        let offset = min_val;
        
        let values = vector
            .iter()
            .map(|&val| ((val - offset) / scale).round().clamp(0.0, 255.0) as u8)
            .collect();
        
        Self { values, scale, offset }
    }
    
    /// Reconstruct approximate f32 vector
    pub fn to_f32(&self) -> Vec<f32> {
        self.values
            .iter()
            .map(|&val| (val as f32) * self.scale + self.offset)
            .collect()
    }
    
    /// Compute cosine similarity without full reconstruction
    pub fn cosine_similarity(&self, other: &Self) -> f32 {
        if self.values.len() != other.values.len() {
            return 0.0;
        }
        
        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;
        
        for (&a, &b) in self.values.iter().zip(other.values.iter()) {
            let val_a = (a as f32) * self.scale + self.offset;
            let val_b = (b as f32) * other.scale + other.offset;
            
            dot_product += val_a * val_b;
            norm_a += val_a * val_a;
            norm_b += val_b * val_b;
        }
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm_a.sqrt() * norm_b.sqrt())
    }
}

/// Product quantization for extreme compression (97% reduction)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductQuantizedVector {
    /// Subvector assignments to centroids
    pub assignments: Vec<u8>,
    /// Number of subvectors
    pub num_subvectors: usize,
    /// Codebook for reconstruction
    pub codebook_id: String,
}

/// Codebook for product quantization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductQuantizationCodebook {
    /// Centroids for each subvector (subvector_idx -> centroid_idx -> centroid)
    pub centroids: Vec<Vec<Vec<f32>>>,
    /// Dimension of each subvector
    pub subvector_dim: usize,
    /// Number of centroids per subvector
    pub num_centroids: usize,
}

impl ProductQuantizationCodebook {
    /// Create new codebook with k-means clustering
    pub fn train(vectors: &[Vec<f32>], num_subvectors: usize, num_centroids: usize) -> Result<Self> {
        if vectors.is_empty() {
            anyhow::bail!("Cannot train on empty vector set");
        }
        
        let dim = vectors[0].len();
        let subvector_dim = dim / num_subvectors;
        
        if dim % num_subvectors != 0 {
            anyhow::bail!("Vector dimension {} not divisible by num_subvectors {}", dim, num_subvectors);
        }
        
        let mut centroids = Vec::with_capacity(num_subvectors);
        
        // Train codebook for each subvector
        for sv_idx in 0..num_subvectors {
            let start_dim = sv_idx * subvector_dim;
            let end_dim = start_dim + subvector_dim;
            
            // Extract subvectors
            let subvectors: Vec<Vec<f32>> = vectors
                .iter()
                .map(|v| v[start_dim..end_dim].to_vec())
                .collect();
            
            // Simple k-means clustering (can be optimized with kmeans++ or GPU)
            let sv_centroids = Self::kmeans_clustering(&subvectors, num_centroids)?;
            centroids.push(sv_centroids);
        }
        
        Ok(Self {
            centroids,
            subvector_dim,
            num_centroids,
        })
    }
    
    /// Simple k-means clustering implementation
    fn kmeans_clustering(vectors: &[Vec<f32>], k: usize) -> Result<Vec<Vec<f32>>> {
        if vectors.is_empty() || k == 0 {
            anyhow::bail!("Invalid parameters for k-means");
        }
        
        let dim = vectors[0].len();
        let mut centroids = Vec::with_capacity(k);
        
        // Initialize centroids randomly
        for i in 0..k {
            if i < vectors.len() {
                centroids.push(vectors[i].clone());
            } else {
                // Fallback: duplicate first centroid
                centroids.push(vectors[0].clone());
            }
        }
        
        // Simple k-means iterations (can be improved)
        for _iteration in 0..10 {
            let mut new_centroids = vec![vec![0.0; dim]; k];
            let mut counts = vec![0; k];
            
            // Assign points to centroids
            for vector in vectors {
                let closest_idx = Self::find_closest_centroid(vector, &centroids);
                
                for (i, &val) in vector.iter().enumerate() {
                    new_centroids[closest_idx][i] += val;
                }
                counts[closest_idx] += 1;
            }
            
            // Update centroids
            for (i, count) in counts.iter().enumerate() {
                if *count > 0 {
                    for j in 0..dim {
                        new_centroids[i][j] /= *count as f32;
                    }
                    centroids[i] = new_centroids[i].clone();
                }
            }
        }
        
        Ok(centroids)
    }
    
    fn find_closest_centroid(vector: &[f32], centroids: &[Vec<f32>]) -> usize {
        let mut best_idx = 0;
        let mut best_dist = f32::INFINITY;
        
        for (idx, centroid) in centroids.iter().enumerate() {
            let dist = Self::euclidean_distance(vector, centroid);
            if dist < best_dist {
                best_dist = dist;
                best_idx = idx;
            }
        }
        
        best_idx
    }
    
    fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y) * (x - y))
            .sum::<f32>()
            .sqrt()
    }
    
    /// Quantize vector using this codebook
    pub fn quantize(&self, vector: &[f32]) -> ProductQuantizedVector {
        let mut assignments = Vec::with_capacity(self.centroids.len());
        
        for (sv_idx, sv_centroids) in self.centroids.iter().enumerate() {
            let start_dim = sv_idx * self.subvector_dim;
            let end_dim = start_dim + self.subvector_dim;
            let subvector = &vector[start_dim..end_dim];
            
            let closest_idx = Self::find_closest_centroid(subvector, sv_centroids);
            assignments.push(closest_idx as u8);
        }
        
        ProductQuantizedVector {
            assignments,
            num_subvectors: self.centroids.len(),
            codebook_id: "default".to_string(), // TODO: implement codebook management
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quantized_vector_compression() {
        let original = vec![0.1, 0.5, -0.3, 0.8, -0.1];
        let quantized = QuantizedVector::from_f32(&original);
        let reconstructed = quantized.to_f32();
        
        // Should be roughly similar
        for (orig, recon) in original.iter().zip(reconstructed.iter()) {
            assert!((orig - recon).abs() < 0.1);
        }
        
        // Memory usage verification
        let original_size = original.len() * 4; // f32 = 4 bytes
        let quantized_size = quantized.values.len() + 8; // u8 + scale + offset
        
        assert!(quantized_size < original_size);
    }
    
    #[test]
    fn test_product_quantization() {
        let vectors = vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![2.0, 3.0, 4.0, 5.0],
            vec![0.0, 1.0, 2.0, 3.0],
        ];
        
        let codebook = ProductQuantizationCodebook::train(&vectors, 2, 2).unwrap();
        let quantized = codebook.quantize(&vectors[0]);
        
        assert_eq!(quantized.assignments.len(), 2);
        assert_eq!(quantized.num_subvectors, 2);
    }
}