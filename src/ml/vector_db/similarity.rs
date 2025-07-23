/*! Similarity and Distance Metrics
 * Optimized implementations for vector similarity computation
 */

use anyhow::Result;

/// Similarity metrics for vectors
pub trait SimilarityMetric: Send + Sync {
    /// Compute similarity between two vectors (higher = more similar)
    fn similarity(&self, a: &[f32], b: &[f32]) -> Result<f32>;
    
    /// Compute distance between two vectors (lower = more similar)
    fn distance(&self, a: &[f32], b: &[f32]) -> Result<f32>;
}

/// Cosine similarity metric
#[derive(Clone, Debug, Default)]
pub struct CosineSimilarity;

impl SimilarityMetric for CosineSimilarity {
    fn similarity(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            anyhow::bail!("Vector dimensions don't match: {} vs {}", a.len(), b.len());
        }
        
        if a.is_empty() {
            return Ok(0.0);
        }
        
        // Compute dot product
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        
        // Compute magnitudes
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        // Avoid division by zero
        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return Ok(0.0);
        }
        
        let similarity = dot_product / (magnitude_a * magnitude_b);
        
        // Clamp to [-1, 1] to handle floating point errors
        Ok(similarity.clamp(-1.0, 1.0))
    }
    
    fn distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        let similarity = self.similarity(a, b)?;
        // Convert similarity [-1, 1] to distance [0, 2]
        Ok(1.0 - similarity)
    }
}

/// Euclidean distance metric
#[derive(Clone, Debug, Default)]
pub struct EuclideanDistance;

impl SimilarityMetric for EuclideanDistance {
    fn similarity(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        let distance = self.distance(a, b)?;
        // Convert distance to similarity using exponential decay
        Ok((-distance).exp())
    }
    
    fn distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            anyhow::bail!("Vector dimensions don't match: {} vs {}", a.len(), b.len());
        }
        
        let squared_distance: f32 = a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y) * (x - y))
            .sum();
        
        Ok(squared_distance.sqrt())
    }
}

/// Manhattan (L1) distance metric
#[derive(Clone, Debug, Default)]
pub struct ManhattanDistance;

impl SimilarityMetric for ManhattanDistance {
    fn similarity(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        let distance = self.distance(a, b)?;
        // Convert distance to similarity
        Ok(1.0 / (1.0 + distance))
    }
    
    fn distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            anyhow::bail!("Vector dimensions don't match: {} vs {}", a.len(), b.len());
        }
        
        let distance: f32 = a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).abs())
            .sum();
        
        Ok(distance)
    }
}

/// Dot product similarity (for normalized vectors)
#[derive(Clone, Debug, Default)]
pub struct DotProductSimilarity;

impl SimilarityMetric for DotProductSimilarity {
    fn similarity(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            anyhow::bail!("Vector dimensions don't match: {} vs {}", a.len(), b.len());
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        Ok(dot_product)
    }
    
    fn distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        let similarity = self.similarity(a, b)?;
        // For normalized vectors, distance = 2 - 2*similarity
        Ok(2.0 - 2.0 * similarity)
    }
}

/// Jaccard similarity (for binary/sparse vectors)
#[derive(Clone, Debug, Default)]
pub struct JaccardSimilarity {
    pub threshold: f32,
}

impl JaccardSimilarity {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}

impl SimilarityMetric for JaccardSimilarity {
    fn similarity(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            anyhow::bail!("Vector dimensions don't match: {} vs {}", a.len(), b.len());
        }
        
        let mut intersection = 0;
        let mut union = 0;
        
        for (x, y) in a.iter().zip(b.iter()) {
            let a_active = *x > self.threshold;
            let b_active = *y > self.threshold;
            
            if a_active && b_active {
                intersection += 1;
            }
            if a_active || b_active {
                union += 1;
            }
        }
        
        if union == 0 {
            Ok(1.0) // Both vectors are all zeros
        } else {
            Ok(intersection as f32 / union as f32)
        }
    }
    
    fn distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        let similarity = self.similarity(a, b)?;
        Ok(1.0 - similarity)
    }
}

/// Batch similarity computation utilities
pub struct BatchSimilarity;

impl BatchSimilarity {
    /// Compute similarities between query and multiple vectors
    pub fn compute_similarities<M>(
        metric: &M,
        query: &[f32],
        vectors: &[&[f32]],
    ) -> Result<Vec<f32>>
    where
        M: SimilarityMetric,
    {
        vectors
            .iter()
            .map(|v| metric.similarity(query, v))
            .collect()
    }
    
    /// Find top-k most similar vectors
    pub fn top_k_similar<M>(
        metric: &M,
        query: &[f32],
        vectors: &[(String, &[f32])],
        k: usize,
    ) -> Result<Vec<(String, f32)>>
    where
        M: SimilarityMetric,
    {
        let mut similarities: Vec<(String, f32)> = vectors
            .iter()
            .map(|(id, v)| {
                let sim = metric.similarity(query, v)?;
                Ok((id.clone(), sim))
            })
            .collect::<Result<Vec<_>>>()?;
        
        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top k
        similarities.truncate(k);
        
        Ok(similarities)
    }
    
    /// Compute similarity matrix for all pairs
    pub fn similarity_matrix<M>(
        metric: &M,
        vectors: &[&[f32]],
    ) -> Result<Vec<Vec<f32>>>
    where
        M: SimilarityMetric,
    {
        let n = vectors.len();
        let mut matrix = vec![vec![0.0; n]; n];
        
        for i in 0..n {
            matrix[i][i] = 1.0; // Self-similarity
            for j in (i + 1)..n {
                let similarity = metric.similarity(vectors[i], vectors[j])?;
                matrix[i][j] = similarity;
                matrix[j][i] = similarity; // Symmetric
            }
        }
        
        Ok(matrix)
    }
}

/// Vector normalization utilities
pub struct VectorNorm;

impl VectorNorm {
    /// L2 normalize vector (unit vector)
    pub fn l2_normalize(vector: &mut [f32]) {
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in vector {
                *val /= magnitude;
            }
        }
    }
    
    /// L1 normalize vector (sum = 1)
    pub fn l1_normalize(vector: &mut [f32]) {
        let sum: f32 = vector.iter().map(|x| x.abs()).sum();
        if sum > 0.0 {
            for val in vector {
                *val /= sum;
            }
        }
    }
    
    /// Min-max normalize vector to [0, 1]
    pub fn minmax_normalize(vector: &mut [f32]) {
        if vector.is_empty() {
            return;
        }
        
        let min_val = *vector.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_val = *vector.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        
        let range = max_val - min_val;
        if range > 0.0 {
            for val in vector {
                *val = (*val - min_val) / range;
            }
        }
    }
    
    /// Z-score normalize vector (mean=0, std=1)
    pub fn zscore_normalize(vector: &mut [f32]) {
        if vector.is_empty() {
            return;
        }
        
        let mean: f32 = vector.iter().sum::<f32>() / vector.len() as f32;
        let variance: f32 = vector.iter()
            .map(|x| (x - mean) * (x - mean))
            .sum::<f32>() / vector.len() as f32;
        let std_dev = variance.sqrt();
        
        if std_dev > 0.0 {
            for val in vector {
                *val = (*val - mean) / std_dev;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cosine_similarity() {
        let metric = CosineSimilarity;
        
        // Identical vectors
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = metric.similarity(&a, &b).unwrap();
        assert!((sim - 1.0).abs() < 1e-6);
        
        // Orthogonal vectors
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = metric.similarity(&a, &b).unwrap();
        assert!(sim.abs() < 1e-6);
        
        // Opposite vectors
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = metric.similarity(&a, &b).unwrap();
        assert!((sim + 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_euclidean_distance() {
        let metric = EuclideanDistance;
        
        // Identical vectors
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let dist = metric.distance(&a, &b).unwrap();
        assert!(dist.abs() < 1e-6);
        
        // Known distance
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let dist = metric.distance(&a, &b).unwrap();
        assert!((dist - 5.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_vector_normalization() {
        let mut vector = vec![3.0, 4.0, 0.0];
        VectorNorm::l2_normalize(&mut vector);
        
        // Check unit vector
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 1e-6);
        
        // Check proportions maintained
        assert!((vector[0] - 0.6).abs() < 1e-6);
        assert!((vector[1] - 0.8).abs() < 1e-6);
        assert!(vector[2].abs() < 1e-6);
    }
    
    #[test]
    fn test_batch_similarity() {
        let metric = CosineSimilarity;
        let query = vec![1.0, 0.0, 0.0];
        
        let vec_a = vec![1.0, 0.0, 0.0];
        let vec_b = vec![0.0, 1.0, 0.0];
        let vec_c = vec![0.5, 0.5, 0.0];
        
        let vectors = vec![
            ("a".to_string(), vec_a.as_slice()),
            ("b".to_string(), vec_b.as_slice()),
            ("c".to_string(), vec_c.as_slice()),
        ];
        
        let results = BatchSimilarity::top_k_similar(&metric, &query, &vectors, 2).unwrap();
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "a"); // Most similar
        assert!(results[0].1 > results[1].1); // Decreasing similarity
    }
}