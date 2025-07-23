use anyhow::Result;
use tree_sitter::{Parser, Node, Tree};
use crate::types::{FunctionInfo, ClassInfo, ComponentInfo, ServiceInfo, PipeInfo, ParameterInfo, PropertyInfo, LocationInfo};

#[derive(Debug, Clone)]
pub struct TypeScriptElement {
    pub kind: String,
    pub name: String,
    pub signature: String,
    pub modifiers: Vec<String>,
    pub location: String,
}

pub struct TypeScriptASTAnalyzer {
    parser: Parser,
}

impl TypeScriptASTAnalyzer {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
        parser.set_language(&language.into())?;
        
        Ok(TypeScriptASTAnalyzer { parser })
    }

    pub fn parse_file(&mut self, content: &str) -> Result<Tree> {
        self.parser.parse(content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse TypeScript content"))
    }

    pub fn extract_elements(&self, tree: &Tree, source_code: &str) -> Vec<TypeScriptElement> {
        let mut elements = Vec::new();
        let source_bytes = source_code.as_bytes();
        self.extract_elements_recursive(tree.root_node(), source_bytes, &mut elements);
        elements
    }

    pub fn extract_functions(&self, tree: &Tree, source_code: &str) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        let source_bytes = source_code.as_bytes();
        self.extract_functions_recursive(tree.root_node(), source_bytes, &mut functions);
        functions
    }

    pub fn extract_classes(&self, tree: &Tree, source_code: &str) -> Vec<ClassInfo> {
        let mut classes = Vec::new();
        let source_bytes = source_code.as_bytes();
        self.extract_classes_recursive(tree.root_node(), source_bytes, &mut classes);
        classes
    }

    pub fn extract_component_info(&self, tree: &Tree, source_code: &str) -> Option<ComponentInfo> {
        let source_bytes = source_code.as_bytes();
        self.find_component_info(tree.root_node(), source_bytes)
    }

    pub fn extract_service_info(&self, tree: &Tree, source_code: &str) -> Option<ServiceInfo> {
        let source_bytes = source_code.as_bytes();
        self.find_service_info(tree.root_node(), source_bytes)
    }

    pub fn extract_pipe_info(&self, tree: &Tree, source_code: &str) -> Option<PipeInfo> {
        let source_bytes = source_code.as_bytes();
        self.find_pipe_info(tree.root_node(), source_bytes)
    }

    fn extract_elements_recursive(&self, node: Node, source_code: &[u8], elements: &mut Vec<TypeScriptElement>) {
        match node.kind() {
            "interface_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(name_node, source_code);
                    let location = format!("{}:{}", node.start_position().row + 1, node.start_position().column + 1);
                    
                    elements.push(TypeScriptElement {
                        kind: "Interface".to_string(),
                        name: name.clone(),
                        signature: format!("interface {}", name),
                        modifiers: vec![],
                        location,
                    });

                    // Extract interface properties
                    if let Some(body) = node.child_by_field_name("body") {
                        self.extract_interface_properties(body, source_code, elements);
                    }
                }
            }
            
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(name_node, source_code);
                    let location = format!("{}:{}", node.start_position().row + 1, node.start_position().column + 1);
                    
                    elements.push(TypeScriptElement {
                        kind: "Class".to_string(),
                        name: name.clone(),
                        signature: format!("class {}", name),
                        modifiers: vec![],
                        location,
                    });

                    // Extract class members
                    if let Some(body) = node.child_by_field_name("body") {
                        self.extract_class_members(body, source_code, elements);
                    }
                }
            }
            
            "function_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(name_node, source_code);
                    let params = if let Some(params_node) = node.child_by_field_name("parameters") {
                        self.node_text(params_node, source_code)
                    } else {
                        "()".to_string()
                    };
                    let return_type = if let Some(return_node) = node.child_by_field_name("return_type") {
                        let type_text = self.node_text(return_node, source_code);
                        type_text.trim_start_matches(':').trim_start().to_string()
                    } else {
                        "".to_string()
                    };
                    let modifiers = self.extract_modifiers(node);
                    let location = format!("{}:{}", node.start_position().row + 1, node.start_position().column + 1);
                    
                    elements.push(TypeScriptElement {
                        kind: "Function".to_string(),
                        name: name.clone(),
                        signature: format!("{}{}{}", name, params, if return_type.is_empty() { "".to_string() } else { format!(": {}", return_type) }),
                        modifiers,
                        location,
                    });
                }
            }
            
            "variable_declaration" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "variable_declarator" {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let name = self.node_text(name_node, source_code);
                            let var_type = if let Some(type_node) = child.child_by_field_name("type") {
                                self.node_text(type_node, source_code)
                            } else {
                                "auto".to_string()
                            };
                            let location = format!("{}:{}", child.start_position().row + 1, child.start_position().column + 1);
                            
                            elements.push(TypeScriptElement {
                                kind: "Variable".to_string(),
                                name: name.clone(),
                                signature: format!("{}: {}", name, var_type),
                                modifiers: vec![],
                                location,
                            });
                        }
                    }
                }
            }
            
            "type_alias_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(name_node, source_code);
                    let type_def = if let Some(value_node) = node.child_by_field_name("value") {
                        let type_text = self.node_text(value_node, source_code);
                        if type_text.len() > 30 {
                            self.truncate_str(&type_text, 27)
                        } else {
                            type_text
                        }
                    } else {
                        "unknown".to_string()
                    };
                    let location = format!("{}:{}", node.start_position().row + 1, node.start_position().column + 1);
                    
                    elements.push(TypeScriptElement {
                        kind: "Type".to_string(),
                        name: name.clone(),
                        signature: format!("{} = {}", name, type_def),
                        modifiers: vec![],
                        location,
                    });
                }
            }
            
            "enum_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(name_node, source_code);
                    let location = format!("{}:{}", node.start_position().row + 1, node.start_position().column + 1);
                    
                    elements.push(TypeScriptElement {
                        kind: "Enum".to_string(),
                        name: name.clone(),
                        signature: format!("enum {}", name),
                        modifiers: vec![],
                        location,
                    });
                }
            }
            
            _ => {
                // Recursively process children
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_elements_recursive(child, source_code, elements);
                }
            }
        }
    }

    fn extract_functions_recursive(&self, node: Node, source_code: &[u8], functions: &mut Vec<FunctionInfo>) {
        match node.kind() {
            "function_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(name_node, source_code);
                    let parameters = if let Some(params_node) = node.child_by_field_name("parameters") {
                        self.extract_parameter_info_list(params_node, source_code)
                    } else {
                        vec![]
                    };
                    let return_type = if let Some(return_node) = node.child_by_field_name("return_type") {
                        let type_text = self.node_text(return_node, source_code);
                        type_text.trim_start_matches(':').trim_start().to_string()
                    } else {
                        "void".to_string()
                    };
                    let is_async = self.node_text(node, source_code).contains("async");
                    let modifiers = self.extract_modifiers(node);
                    let location = LocationInfo {
                        line: node.start_position().row + 1,
                        column: node.start_position().column + 1,
                    };
                    
                    functions.push(FunctionInfo {
                        name,
                        parameters,
                        return_type,
                        is_async,
                        modifiers,
                        location,
                        description: None,
                    });
                }
            }
            "method_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(name_node, source_code);
                    let parameters = if let Some(params_node) = node.child_by_field_name("parameters") {
                        self.extract_parameter_info_list(params_node, source_code)
                    } else {
                        vec![]
                    };
                    let return_type = if let Some(return_node) = node.child_by_field_name("return_type") {
                        let type_text = self.node_text(return_node, source_code);
                        type_text.trim_start_matches(':').trim_start().to_string()
                    } else {
                        "void".to_string()
                    };
                    let is_async = self.node_text(node, source_code).contains("async");
                    let modifiers = self.extract_modifiers(node);
                    let location = LocationInfo {
                        line: node.start_position().row + 1,
                        column: node.start_position().column + 1,
                    };
                    
                    functions.push(FunctionInfo {
                        name,
                        parameters,
                        return_type,
                        is_async,
                        modifiers,
                        location,
                        description: None,
                    });
                }
            }
            _ => {
                // Recursively process children
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_functions_recursive(child, source_code, functions);
                }
            }
        }
    }

    fn extract_classes_recursive(&self, node: Node, source_code: &[u8], classes: &mut Vec<ClassInfo>) {
        match node.kind() {
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(name_node, source_code);
                    let mut methods = Vec::new();
                    let mut properties = Vec::new();
                    let extends = None; // TODO: Extract extends clause
                    let implements = Vec::new(); // TODO: Extract implements clause
                    let modifiers = self.extract_modifiers(node);
                    let location = LocationInfo {
                        line: node.start_position().row + 1,
                        column: node.start_position().column + 1,
                    };
                    
                    // Extract class members
                    if let Some(body) = node.child_by_field_name("body") {
                        self.extract_class_content(body, source_code, &mut methods, &mut properties);
                    }
                    
                    classes.push(ClassInfo {
                        name,
                        methods,
                        properties,
                        extends,
                        implements,
                        modifiers,
                        location,
                    });
                }
            }
            _ => {
                // Recursively process children
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_classes_recursive(child, source_code, classes);
                }
            }
        }
    }

    fn extract_class_content(&self, body_node: Node, source_code: &[u8], methods: &mut Vec<FunctionInfo>, properties: &mut Vec<PropertyInfo>) {
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            match child.kind() {
                "method_definition" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source_code);
                        let parameters = if let Some(params_node) = child.child_by_field_name("parameters") {
                            self.extract_parameter_info_list(params_node, source_code)
                        } else {
                            vec![]
                        };
                        let return_type = if let Some(return_node) = child.child_by_field_name("return_type") {
                            let type_text = self.node_text(return_node, source_code);
                            type_text.trim_start_matches(':').trim_start().to_string()
                        } else {
                            "void".to_string()
                        };
                        let is_async = self.node_text(child, source_code).contains("async");
                        let modifiers = self.extract_modifiers(child);
                        let location = LocationInfo {
                            line: child.start_position().row + 1,
                            column: child.start_position().column + 1,
                        };
                        
                        methods.push(FunctionInfo {
                            name,
                            parameters,
                            return_type,
                            is_async,
                            modifiers,
                            location,
                            description: None,
                        });
                    }
                }
                "property_definition" | "field_definition" | "public_field_definition" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source_code);
                        let prop_type = if let Some(type_node) = child.child_by_field_name("type") {
                            let type_text = self.node_text(type_node, source_code);
                            type_text.trim_start_matches(':').trim_start().to_string()
                        } else {
                            "any".to_string()
                        };
                        let modifiers = self.extract_modifiers(child);
                        let location = LocationInfo {
                            line: child.start_position().row + 1,
                            column: child.start_position().column + 1,
                        };
                        let initial_value = if let Some(value_node) = child.child_by_field_name("value") {
                            Some(self.node_text(value_node, source_code))
                        } else {
                            None
                        };
                        
                        properties.push(PropertyInfo {
                            name,
                            prop_type,
                            modifiers,
                            location,
                            initial_value,
                        });
                    }
                }
                "lexical_declaration" => {
                    // Handle private properties like 'private http = inject(HttpClient)'
                    let mut decl_cursor = child.walk();
                    for decl_child in child.children(&mut decl_cursor) {
                        if decl_child.kind() == "variable_declarator" {
                            if let Some(name_node) = decl_child.child_by_field_name("name") {
                                let name = self.node_text(name_node, source_code);
                                let (prop_type, initial_value) = if let Some(value_node) = decl_child.child_by_field_name("value") {
                                    let value_text = self.node_text(value_node, source_code);
                                    let inferred_type = self.infer_type_from_value(&value_text);
                                    (inferred_type, Some(value_text))
                                } else {
                                    ("any".to_string(), None)
                                };
                                let modifiers = self.extract_modifiers(child);
                                let location = LocationInfo {
                                    line: decl_child.start_position().row + 1,
                                    column: decl_child.start_position().column + 1,
                                };
                                
                                properties.push(PropertyInfo {
                                    name,
                                    prop_type,
                                    modifiers,
                                    location,
                                    initial_value,
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_interface_properties(&self, body_node: Node, source_code: &[u8], elements: &mut Vec<TypeScriptElement>) {
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            if child.kind() == "property_signature" {
                if let Some(prop_name_node) = child.child_by_field_name("name") {
                    let prop_name = self.node_text(prop_name_node, source_code);
                    let prop_type = if let Some(type_node) = child.child_by_field_name("type") {
                        let type_text = self.node_text(type_node, source_code);
                        type_text.trim_start_matches(':').trim_start().to_string()
                    } else {
                        "any".to_string()
                    };
                    let optional = if self.node_text(child, source_code).contains('?') { "?" } else { "" };
                    let prop_location = format!("{}:{}", child.start_position().row + 1, child.start_position().column + 1);
                    
                    elements.push(TypeScriptElement {
                        kind: "Property".to_string(),
                        name: prop_name.clone(),
                        signature: format!("{}{}: {}", prop_name, optional, prop_type),
                        modifiers: vec![],
                        location: prop_location,
                    });
                }
            }
        }
    }

    fn extract_class_members(&self, body_node: Node, source_code: &[u8], elements: &mut Vec<TypeScriptElement>) {
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            match child.kind() {
                "method_definition" => {
                    if let Some(method_name_node) = child.child_by_field_name("name") {
                        let method_name = self.node_text(method_name_node, source_code);
                        let params = if let Some(params_node) = child.child_by_field_name("parameters") {
                            self.node_text(params_node, source_code)
                        } else {
                            "()".to_string()
                        };
                        let return_type = if let Some(return_node) = child.child_by_field_name("return_type") {
                            let type_text = self.node_text(return_node, source_code);
                            type_text.trim_start_matches(':').trim_start().to_string()
                        } else {
                            "".to_string()
                        };
                        let modifiers = self.extract_modifiers(child);
                        let method_location = format!("{}:{}", child.start_position().row + 1, child.start_position().column + 1);
                        
                        elements.push(TypeScriptElement {
                            kind: "Method".to_string(),
                            name: method_name.clone(),
                            signature: format!("{}{}{}", method_name, params, if return_type.is_empty() { "".to_string() } else { format!(": {}", return_type) }),
                            modifiers,
                            location: method_location,
                        });
                    }
                }
                "property_definition" | "field_definition" | "public_field_definition" => {
                    if let Some(prop_name_node) = child.child_by_field_name("name") {
                        let prop_name = self.node_text(prop_name_node, source_code);
                        let prop_type = if let Some(type_node) = child.child_by_field_name("type") {
                            self.node_text(type_node, source_code)
                        } else if let Some(value_node) = child.child_by_field_name("value") {
                            let value_text = self.node_text(value_node, source_code);
                            self.infer_type_from_value(&value_text)
                        } else {
                            "auto".to_string()
                        };
                        let modifiers = self.extract_modifiers(child);
                        let prop_location = format!("{}:{}", child.start_position().row + 1, child.start_position().column + 1);
                        
                        elements.push(TypeScriptElement {
                            kind: "Property".to_string(),
                            name: prop_name.clone(),
                            signature: format!("{}: {}", prop_name, prop_type),
                            modifiers,
                            location: prop_location,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_parameter_info_list(&self, params_node: Node, source_code: &[u8]) -> Vec<ParameterInfo> {
        let mut parameters = Vec::new();
        let mut cursor = params_node.walk();
        
        for child in params_node.children(&mut cursor) {
            if child.kind() == "required_parameter" || child.kind() == "optional_parameter" {
                if let Some(pattern_node) = child.child_by_field_name("pattern") {
                    let param_name = self.node_text(pattern_node, source_code);
                    let param_type = if let Some(type_node) = child.child_by_field_name("type") {
                        let type_text = self.node_text(type_node, source_code);
                        type_text.trim_start_matches(':').trim_start().to_string()
                    } else {
                        "any".to_string()
                    };
                    let is_optional = child.kind() == "optional_parameter";
                    let default_value = if let Some(default_node) = child.child_by_field_name("value") {
                        Some(self.node_text(default_node, source_code))
                    } else {
                        None
                    };
                    
                    parameters.push(ParameterInfo {
                        name: param_name,
                        param_type,
                        is_optional,
                        default_value,
                    });
                }
            }
        }
        
        parameters
    }

    fn extract_parameter_list(&self, params_node: Node, source_code: &[u8]) -> Vec<String> {
        let mut parameters = Vec::new();
        let mut cursor = params_node.walk();
        
        for child in params_node.children(&mut cursor) {
            if child.kind() == "required_parameter" || child.kind() == "optional_parameter" {
                if let Some(pattern_node) = child.child_by_field_name("pattern") {
                    let param_name = self.node_text(pattern_node, source_code);
                    let param_type = if let Some(type_node) = child.child_by_field_name("type") {
                        let type_text = self.node_text(type_node, source_code);
                        type_text.trim_start_matches(':').trim_start().to_string()
                    } else {
                        "any".to_string()
                    };
                    parameters.push(format!("{}: {}", param_name, param_type));
                }
            }
        }
        
        parameters
    }

    fn find_component_info(&self, node: Node, source_code: &[u8]) -> Option<ComponentInfo> {
        // Look for @Component decorator
        if self.node_text(node, source_code).contains("@Component") {
            // Extract component information
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "class_declaration" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source_code);
                        return Some(ComponentInfo {
                            name,
                            selector: self.extract_selector(node, source_code),
                            inputs: self.extract_input_properties(child, source_code),
                            outputs: self.extract_output_properties(child, source_code),
                            lifecycle: self.extract_lifecycle(child, source_code),
                            template_summary: "Component template".to_string(),
                            location: LocationInfo {
                                line: child.start_position().row + 1,
                                column: child.start_position().column + 1,
                            },
                        });
                    }
                }
            }
        }
        
        // Recursively search children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(component) = self.find_component_info(child, source_code) {
                return Some(component);
            }
        }
        
        None
    }

    fn find_service_info(&self, node: Node, source_code: &[u8]) -> Option<ServiceInfo> {
        // Look for @Injectable decorator
        if self.node_text(node, source_code).contains("@Injectable") {
            // Extract service information
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "class_declaration" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source_code);
                        let mut methods = Vec::new();
                        
                        // Extract methods
                        if let Some(body) = child.child_by_field_name("body") {
                            self.extract_class_content(body, source_code, &mut methods, &mut Vec::new());
                        }
                        
                        return Some(ServiceInfo {
                            name,
                            injectable: true,
                            provided_in: None, // TODO: Extract from @Injectable decorator
                            scope: crate::types::ServiceScope::Root, // Default scope
                            dependencies: self.extract_service_dependencies(child, source_code),
                            methods,
                            location: LocationInfo {
                                line: child.start_position().row + 1,
                                column: child.start_position().column + 1,
                            },
                        });
                    }
                }
            }
        }
        
        // Recursively search children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(service) = self.find_service_info(child, source_code) {
                return Some(service);
            }
        }
        
        None
    }

    fn find_pipe_info(&self, node: Node, source_code: &[u8]) -> Option<PipeInfo> {
        // Look for @Pipe decorator
        if self.node_text(node, source_code).contains("@Pipe") {
            // Extract pipe information
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "class_declaration" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source_code);
                        
                        // Extract transform method
                        let transform_method = self.find_transform_method(child, source_code);
                        
                        return Some(PipeInfo {
                            name,
                            transform_method: transform_method.unwrap_or_else(|| {
                                FunctionInfo {
                                    name: "transform".to_string(),
                                    parameters: vec![],
                                    return_type: "any".to_string(),
                                    is_async: false,
                                    modifiers: vec![],
                                    location: LocationInfo { line: 1, column: 1 },
                                    description: None,
                                }
                            }),
                            is_pure: self.extract_pipe_pure_flag(node, source_code),
                            is_standalone: self.extract_pipe_standalone_flag(node, source_code),
                            dependencies: self.extract_pipe_dependencies(child, source_code),
                            location: LocationInfo {
                                line: child.start_position().row + 1,
                                column: child.start_position().column + 1,
                            },
                        });
                    }
                }
            }
        }
        
        // Recursively search children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(pipe) = self.find_pipe_info(child, source_code) {
                return Some(pipe);
            }
        }
        
        None
    }

    fn find_transform_method(&self, class_node: Node, source_code: &[u8]) -> Option<FunctionInfo> {
        if let Some(body) = class_node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "method_definition" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source_code);
                        if name == "transform" {
                            let parameters = if let Some(params_node) = child.child_by_field_name("parameters") {
                                self.extract_parameter_info_list(params_node, source_code)
                            } else {
                                vec![]
                            };
                            let return_type = if let Some(return_node) = child.child_by_field_name("return_type") {
                                let type_text = self.node_text(return_node, source_code);
                                type_text.trim_start_matches(':').trim_start().to_string()
                            } else {
                                "any".to_string()
                            };
                            let is_async = self.node_text(child, source_code).contains("async");
                            let modifiers = self.extract_modifiers(child);
                            let location = LocationInfo {
                                line: child.start_position().row + 1,
                                column: child.start_position().column + 1,
                            };
                            
                            return Some(FunctionInfo {
                                name,
                                parameters,
                                return_type,
                                is_async,
                                modifiers,
                                location,
                                description: Some("Pipe transform method".to_string()),
                            });
                        }
                    }
                }
            }
        }
        None
    }

    fn extract_pipe_pure_flag(&self, node: Node, source_code: &[u8]) -> bool {
        let text = self.node_text(node, source_code);
        if let Some(pipe_start) = text.find("@Pipe") {
            let pipe_section = &text[pipe_start..];
            if let Some(pure_match) = pipe_section.find("pure:") {
                let after_pure = &pipe_section[pure_match + 5..];
                return after_pure.trim_start().starts_with("true");
            }
        }
        // Default is true for Angular pipes
        true
    }

    fn extract_pipe_standalone_flag(&self, node: Node, source_code: &[u8]) -> bool {
        let text = self.node_text(node, source_code);
        if let Some(pipe_start) = text.find("@Pipe") {
            let pipe_section = &text[pipe_start..];
            if let Some(standalone_match) = pipe_section.find("standalone:") {
                let after_standalone = &pipe_section[standalone_match + 11..];
                return after_standalone.trim_start().starts_with("true");
            }
        }
        false
    }

    fn extract_pipe_dependencies(&self, class_node: Node, source_code: &[u8]) -> Vec<ParameterInfo> {
        let mut dependencies = Vec::new();
        
        // Look for constructor parameters in class body
        if let Some(body) = class_node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "method_definition" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source_code);
                        if name == "constructor" {
                            if let Some(params_node) = child.child_by_field_name("parameters") {
                                dependencies.extend(self.extract_parameter_info_list(params_node, source_code));
                            }
                        }
                    }
                }
            }
        }
        
        dependencies
    }

    fn extract_selector(&self, node: Node, source_code: &[u8]) -> String {
        // This is a simplified extraction - in real implementation, you'd parse the decorator
        let text = self.node_text(node, source_code);
        if let Some(start) = text.find("selector:") {
            // Try double quotes first
            if let Some(quote_start) = text[start..].find('"') {
                let quote_start = start + quote_start + 1;
                if let Some(quote_end) = text[quote_start..].find('"') {
                    return text[quote_start..quote_start + quote_end].to_string();
                }
            }
            // Try single quotes
            if let Some(quote_start) = text[start..].find('\'') {
                let quote_start = start + quote_start + 1;
                if let Some(quote_end) = text[quote_start..].find('\'') {
                    return text[quote_start..quote_start + quote_end].to_string();
                }
            }
        }
        "unknown".to_string()
    }

    fn extract_input_properties(&self, node: Node, source_code: &[u8]) -> Vec<PropertyInfo> {
        let mut inputs = Vec::new();
        let text = self.node_text(node, source_code);
        
        // Simple extraction for @Input() decorators
        for (line_num, line) in text.lines().enumerate() {
            if line.contains("@Input()") {
                if let Some(prop_start) = line.find("@Input()") {
                    let after_input = &line[prop_start + 8..].trim();
                    if let Some(prop_name) = after_input.split_whitespace().next() {
                        let clean_name = prop_name.trim_end_matches(':').to_string();
                        
                        // Try to extract type information
                        let prop_type = if let Some(colon_pos) = line.find(':') {
                            let type_part = &line[colon_pos + 1..];
                            if let Some(semicolon_pos) = type_part.find(';') {
                                type_part[..semicolon_pos].trim().to_string()
                            } else {
                                type_part.trim().to_string()
                            }
                        } else {
                            "any".to_string()
                        };
                        
                        inputs.push(PropertyInfo {
                            name: clean_name,
                            prop_type,
                            modifiers: vec!["@Input()".to_string()],
                            location: LocationInfo {
                                line: line_num + 1,
                                column: 1,
                            },
                            initial_value: None,
                        });
                    }
                }
            }
        }
        
        inputs
    }

    fn extract_inputs(&self, node: Node, source_code: &[u8]) -> Vec<String> {
        // Look for @Input() decorators
        let mut inputs = Vec::new();
        let text = self.node_text(node, source_code);
        
        // Simple regex-like extraction (in real implementation, you'd parse the AST)
        for line in text.lines() {
            if line.contains("@Input()") {
                // Extract property name after @Input()
                if let Some(prop_start) = line.find("@Input()") {
                    let after_input = &line[prop_start + 8..].trim();
                    if let Some(prop_name) = after_input.split_whitespace().next() {
                        inputs.push(prop_name.trim_end_matches(':').to_string());
                    }
                }
            }
        }
        
        inputs
    }

    fn extract_output_properties(&self, node: Node, source_code: &[u8]) -> Vec<PropertyInfo> {
        let mut outputs = Vec::new();
        let text = self.node_text(node, source_code);
        
        // Simple extraction for @Output() decorators
        for (line_num, line) in text.lines().enumerate() {
            if line.contains("@Output()") {
                if let Some(prop_start) = line.find("@Output()") {
                    let after_output = &line[prop_start + 9..].trim();
                    if let Some(prop_name) = after_output.split_whitespace().next() {
                        let clean_name = prop_name.trim_end_matches(':').to_string();
                        
                        // Try to extract type information
                        let prop_type = if let Some(colon_pos) = line.find(':') {
                            let type_part = &line[colon_pos + 1..];
                            if let Some(semicolon_pos) = type_part.find(';') {
                                type_part[..semicolon_pos].trim().to_string()
                            } else {
                                type_part.trim().to_string()
                            }
                        } else {
                            "EventEmitter".to_string()
                        };
                        
                        outputs.push(PropertyInfo {
                            name: clean_name,
                            prop_type,
                            modifiers: vec!["@Output()".to_string()],
                            location: LocationInfo {
                                line: line_num + 1,
                                column: 1,
                            },
                            initial_value: None,
                        });
                    }
                }
            }
        }
        
        outputs
    }

    fn extract_outputs(&self, node: Node, source_code: &[u8]) -> Vec<String> {
        // Look for @Output() decorators
        let mut outputs = Vec::new();
        let text = self.node_text(node, source_code);
        
        for line in text.lines() {
            if line.contains("@Output()") {
                // Extract property name after @Output()
                if let Some(prop_start) = line.find("@Output()") {
                    let after_output = &line[prop_start + 9..].trim();
                    if let Some(prop_name) = after_output.split_whitespace().next() {
                        outputs.push(prop_name.trim_end_matches(':').to_string());
                    }
                }
            }
        }
        
        outputs
    }

    fn extract_lifecycle(&self, node: Node, source_code: &[u8]) -> Vec<String> {
        let mut lifecycle = Vec::new();
        let text = self.node_text(node, source_code);
        
        let lifecycle_methods = ["ngOnInit", "ngOnDestroy", "ngOnChanges", "ngDoCheck", "ngAfterContentInit", "ngAfterContentChecked", "ngAfterViewInit", "ngAfterViewChecked"];
        
        for method in lifecycle_methods {
            if text.contains(method) {
                lifecycle.push(method.to_string());
            }
        }
        
        lifecycle
    }

    fn extract_service_dependencies(&self, node: Node, source_code: &[u8]) -> Vec<ParameterInfo> {
        let mut dependencies = Vec::new();
        
        // Look for constructor parameters in class body
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "method_definition" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source_code);
                        if name == "constructor" {
                            if let Some(params_node) = child.child_by_field_name("parameters") {
                                dependencies.extend(self.extract_parameter_info_list(params_node, source_code));
                            }
                        }
                    }
                }
            }
        }
        
        dependencies
    }

    fn extract_dependencies(&self, node: Node, source_code: &[u8]) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Look for constructor parameters
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "method_definition" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.node_text(name_node, source_code);
                    if name == "constructor" {
                        if let Some(params_node) = child.child_by_field_name("parameters") {
                            let params = self.extract_parameter_list(params_node, source_code);
                            dependencies.extend(params);
                        }
                    }
                }
            }
        }
        
        dependencies
    }

    fn extract_modifiers(&self, node: Node) -> Vec<String> {
        let mut modifiers = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "public" | "private" | "protected" | "static" | "readonly" | "abstract" | "async" => {
                    modifiers.push(child.kind().to_string());
                }
                _ => {}
            }
        }
        
        modifiers
    }

    fn infer_type_from_value(&self, value: &str) -> String {
        let trimmed = value.trim();
        
        // Check for common Angular patterns
        if trimmed.starts_with("inject(") || trimmed.starts_with("signal(") {
            if let Some(start) = trimmed.find('(') {
                if let Some(end) = trimmed.rfind(')') {
                    let inner = &trimmed[start + 1..end].trim();
                    if inner.chars().next().map_or(false, |c| c.is_uppercase()) {
                        return inner.to_string();
                    }
                }
            }
            return "auto".to_string();
        }
        
        // Basic type inference
        if trimmed.starts_with('"') || trimmed.starts_with('\'') || trimmed.starts_with('`') {
            return "string".to_string();
        }
        
        if trimmed.parse::<f64>().is_ok() {
            return "number".to_string();
        }
        
        if trimmed == "true" || trimmed == "false" {
            return "boolean".to_string();
        }
        
        if trimmed.starts_with('[') {
            return "array".to_string();
        }
        
        if trimmed.starts_with('{') {
            return "object".to_string();
        }
        
        if trimmed == "null" {
            return "null".to_string();
        }
        
        if trimmed == "undefined" {
            return "undefined".to_string();
        }
        
        "auto".to_string()
    }

    fn node_text(&self, node: Node, source_code: &[u8]) -> String {
        node.utf8_text(source_code).unwrap_or_default().to_string()
    }

    fn truncate_str(&self, s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            return s.to_string();
        }
        
        let mut cut_point = max_len.saturating_sub(3);
        while cut_point > 0 && !s.is_char_boundary(cut_point) {
            cut_point -= 1;
        }
        
        if cut_point == 0 {
            return String::new();
        }
        
        format!("{}...", &s[..cut_point])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_ast_analyzer_creation() {
        let analyzer = TypeScriptASTAnalyzer::new();
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_parse_simple_function() -> Result<()> {
        let mut analyzer = TypeScriptASTAnalyzer::new()?;
        let content = "function test(param: string): number { return 42; }";
        let tree = analyzer.parse_file(content)?;
        let functions = analyzer.extract_functions(&tree, content);
        
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "test");
        assert_eq!(functions[0].parameters.len(), 1);
        assert_eq!(functions[0].parameters[0].name, "param");
        assert_eq!(functions[0].parameters[0].param_type, "string");
        assert!(!functions[0].parameters[0].is_optional);
        assert_eq!(functions[0].return_type, "number");
        assert!(!functions[0].is_async);
        assert!(functions[0].modifiers.is_empty());
        assert_eq!(functions[0].location.line, 1);
        
        Ok(())
    }

    #[test]
    fn test_parse_class_with_methods() -> Result<()> {
        let mut analyzer = TypeScriptASTAnalyzer::new()?;
        let content = r#"
        class TestClass {
            private name: string;
            
            constructor(name: string) {
                this.name = name;
            }
            
            public getName(): string {
                return this.name;
            }
        }
        "#;
        let tree = analyzer.parse_file(content)?;
        let classes = analyzer.extract_classes(&tree, content);
        
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, "TestClass");
        assert_eq!(classes[0].methods.len(), 2); // constructor + getName
        assert_eq!(classes[0].properties.len(), 1);
        
        // Validate property details
        assert_eq!(classes[0].properties[0].name, "name");
        assert_eq!(classes[0].properties[0].prop_type, "string");
        // Note: tree-sitter may not extract 'private' modifier correctly for properties
        // This is a known limitation in the current implementation
        
        // Validate method details
        let get_name_method = classes[0].methods.iter().find(|m| m.name == "getName");
        assert!(get_name_method.is_some());
        let method = get_name_method.unwrap();
        assert_eq!(method.return_type, "string");
        // Note: tree-sitter may not extract 'public' modifier correctly for methods
        // This is a known limitation in the current implementation
        
        Ok(())
    }

    #[test]
    fn test_parse_angular_component() -> Result<()> {
        let mut analyzer = TypeScriptASTAnalyzer::new()?;
        let content = r#"
        @Component({
            selector: 'app-test',
            template: '<p>Test</p>'
        })
        export class TestComponent implements OnInit {
            @Input() data: string;
            @Output() dataChange = new EventEmitter<string>();
            
            ngOnInit() {
                console.log('initialized');
            }
        }
        "#;
        let tree = analyzer.parse_file(content)?;
        let component = analyzer.extract_component_info(&tree, content);
        
        assert!(component.is_some());
        let comp = component.unwrap();
        assert_eq!(comp.name, "TestComponent");
        assert_eq!(comp.selector, "app-test");
        assert!(comp.lifecycle.contains(&"ngOnInit".to_string()));
        
        // Validate inputs
        assert_eq!(comp.inputs.len(), 1);
        assert_eq!(comp.inputs[0].name, "data");
        assert!(comp.inputs[0].modifiers.contains(&"@Input()".to_string()));
        
        // Validate outputs
        assert_eq!(comp.outputs.len(), 1);
        assert_eq!(comp.outputs[0].name, "dataChange");
        assert!(comp.outputs[0].modifiers.contains(&"@Output()".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_parse_async_function() -> Result<()> {
        let mut analyzer = TypeScriptASTAnalyzer::new()?;
        let content = "async function fetchData(url: string): Promise<Response> { return fetch(url); }";
        let tree = analyzer.parse_file(content)?;
        let functions = analyzer.extract_functions(&tree, content);
        
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "fetchData");
        assert!(functions[0].is_async);
        assert_eq!(functions[0].return_type, "Promise<Response>");
        assert!(functions[0].modifiers.contains(&"async".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_parse_interface() -> Result<()> {
        let mut analyzer = TypeScriptASTAnalyzer::new()?;
        let content = r#"
        interface User {
            id: number;
            name: string;
            email?: string;
        }
        "#;
        let tree = analyzer.parse_file(content)?;
        let elements = analyzer.extract_elements(&tree, content);
        
        let interface_elem = elements.iter().find(|e| e.kind == "Interface");
        assert!(interface_elem.is_some());
        let interface = interface_elem.unwrap();
        assert_eq!(interface.name, "User");
        assert_eq!(interface.signature, "interface User");
        
        Ok(())
    }

    #[test]
    fn test_parse_enum() -> Result<()> {
        let mut analyzer = TypeScriptASTAnalyzer::new()?;
        let content = r#"
        enum Status {
            Active,
            Inactive,
            Pending
        }
        "#;
        let tree = analyzer.parse_file(content)?;
        let elements = analyzer.extract_elements(&tree, content);
        
        let enum_elem = elements.iter().find(|e| e.kind == "Enum");
        assert!(enum_elem.is_some());
        let enum_el = enum_elem.unwrap();
        assert_eq!(enum_el.name, "Status");
        assert_eq!(enum_el.signature, "enum Status");
        
        Ok(())
    }

    #[test]
    fn test_parse_angular_service() -> Result<()> {
        let mut analyzer = TypeScriptASTAnalyzer::new()?;
        let content = r#"
        @Injectable({
            providedIn: 'root'
        })
        export class UserService {
            constructor(private http: HttpClient) {}
            
            getUser(id: number): Observable<User> {
                return this.http.get<User>(`/api/users/${id}`);
            }
        }
        "#;
        let tree = analyzer.parse_file(content)?;
        let service = analyzer.extract_service_info(&tree, content);
        
        assert!(service.is_some());
        let svc = service.unwrap();
        assert_eq!(svc.name, "UserService");
        assert!(svc.injectable);
        assert_eq!(svc.methods.len(), 2); // constructor + getUser
        assert_eq!(svc.dependencies.len(), 1);
        assert_eq!(svc.dependencies[0].name, "http");
        assert_eq!(svc.dependencies[0].param_type, "HttpClient");
        
        Ok(())
    }

    #[test]
    fn test_parse_complex_parameters() -> Result<()> {
        let mut analyzer = TypeScriptASTAnalyzer::new()?;
        let content = "function process(data: User[], options?: ProcessOptions, callback: (result: string) => void): Promise<void> {}";
        let tree = analyzer.parse_file(content)?;
        let functions = analyzer.extract_functions(&tree, content);
        
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "process");
        assert_eq!(functions[0].parameters.len(), 3);
        
        // Validate complex parameter types
        assert_eq!(functions[0].parameters[0].name, "data");
        assert_eq!(functions[0].parameters[0].param_type, "User[]");
        assert!(!functions[0].parameters[0].is_optional);
        
        assert_eq!(functions[0].parameters[1].name, "options");
        assert_eq!(functions[0].parameters[1].param_type, "ProcessOptions");
        assert!(functions[0].parameters[1].is_optional);
        
        assert_eq!(functions[0].return_type, "Promise<void>");
        
        Ok(())
    }

    #[test]
    fn test_parse_type_alias() -> Result<()> {
        let mut analyzer = TypeScriptASTAnalyzer::new()?;
        let content = "type StringOrNumber = string | number;";
        let tree = analyzer.parse_file(content)?;
        let elements = analyzer.extract_elements(&tree, content);
        
        let type_elem = elements.iter().find(|e| e.kind == "Type");
        assert!(type_elem.is_some());
        let type_alias = type_elem.unwrap();
        assert_eq!(type_alias.name, "StringOrNumber");
        assert!(type_alias.signature.contains("string | number"));
        
        Ok(())
    }

    #[test]
    fn test_parse_class_with_inheritance() -> Result<()> {
        let mut analyzer = TypeScriptASTAnalyzer::new()?;
        let content = r#"
        export class AdminUser extends User implements Permissions {
            private permissions: string[];
            
            constructor(name: string, permissions: string[]) {
                super(name);
                this.permissions = permissions;
            }
            
            hasPermission(permission: string): boolean {
                return this.permissions.includes(permission);
            }
        }
        "#;
        let tree = analyzer.parse_file(content)?;
        let classes = analyzer.extract_classes(&tree, content);
        
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, "AdminUser");
        assert_eq!(classes[0].methods.len(), 2); // constructor + hasPermission
        assert_eq!(classes[0].properties.len(), 1); // permissions
        
        // Check method details
        let has_permission = classes[0].methods.iter().find(|m| m.name == "hasPermission");
        assert!(has_permission.is_some());
        let method = has_permission.unwrap();
        assert_eq!(method.return_type, "boolean");
        assert_eq!(method.parameters.len(), 1);
        assert_eq!(method.parameters[0].name, "permission");
        assert_eq!(method.parameters[0].param_type, "string");
        
        Ok(())
    }

    #[test]
    fn test_parse_angular_pipe() -> Result<()> {
        let mut analyzer = TypeScriptASTAnalyzer::new()?;
        let content = r#"
        @Pipe({
            name: 'uppercase',
            pure: false,
            standalone: true
        })
        export class UppercasePipe implements PipeTransform {
            constructor(private locale: string) {}
            
            transform(value: string, ...args: any[]): string {
                return value ? value.toUpperCase() : '';
            }
        }
        "#;
        let tree = analyzer.parse_file(content)?;
        let pipe = analyzer.extract_pipe_info(&tree, content);
        
        assert!(pipe.is_some());
        let p = pipe.unwrap();
        assert_eq!(p.name, "UppercasePipe");
        assert!(!p.is_pure); // pure: false
        assert!(p.is_standalone); // standalone: true
        assert_eq!(p.dependencies.len(), 1);
        assert_eq!(p.dependencies[0].name, "locale");
        assert_eq!(p.dependencies[0].param_type, "string");
        assert_eq!(p.transform_method.name, "transform");
        assert_eq!(p.transform_method.parameters.len(), 2); // value + args
        assert_eq!(p.transform_method.return_type, "string");
        
        Ok(())
    }

    #[test]
    fn test_parse_simple_pipe() -> Result<()> {
        let mut analyzer = TypeScriptASTAnalyzer::new()?;
        let content = r#"
        @Pipe({
            name: 'capitalize'
        })
        export class CapitalizePipe implements PipeTransform {
            transform(value: string): string {
                return value.charAt(0).toUpperCase() + value.slice(1);
            }
        }
        "#;
        let tree = analyzer.parse_file(content)?;
        let pipe = analyzer.extract_pipe_info(&tree, content);
        
        assert!(pipe.is_some());
        let p = pipe.unwrap();
        assert_eq!(p.name, "CapitalizePipe");
        assert!(p.is_pure); // default is true
        assert!(!p.is_standalone); // default is false
        assert_eq!(p.dependencies.len(), 0); // no constructor dependencies
        assert_eq!(p.transform_method.name, "transform");
        assert_eq!(p.transform_method.parameters.len(), 1);
        assert_eq!(p.transform_method.parameters[0].name, "value");
        assert_eq!(p.transform_method.parameters[0].param_type, "string");
        assert_eq!(p.transform_method.return_type, "string");
        
        Ok(())
    }

    #[test]
    fn test_pipe_pure_flag_extraction() -> Result<()> {
        let mut analyzer = TypeScriptASTAnalyzer::new()?;
        
        // Test pure: true
        let pure_content = r#"
        @Pipe({
            name: 'test',
            pure: true
        })
        export class TestPipe {}
        "#;
        let tree = analyzer.parse_file(pure_content)?;
        assert!(analyzer.extract_pipe_pure_flag(tree.root_node(), pure_content.as_bytes()));
        
        // Test pure: false
        let impure_content = r#"
        @Pipe({
            name: 'test',
            pure: false
        })
        export class TestPipe {}
        "#;
        let tree = analyzer.parse_file(impure_content)?;
        assert!(!analyzer.extract_pipe_pure_flag(tree.root_node(), impure_content.as_bytes()));
        
        // Test default (should be true)
        let default_content = r#"
        @Pipe({
            name: 'test'
        })
        export class TestPipe {}
        "#;
        let tree = analyzer.parse_file(default_content)?;
        assert!(analyzer.extract_pipe_pure_flag(tree.root_node(), default_content.as_bytes()));
        
        Ok(())
    }

    #[test]
    fn test_pipe_standalone_flag_extraction() -> Result<()> {
        let mut analyzer = TypeScriptASTAnalyzer::new()?;
        
        // Test standalone: true
        let standalone_content = r#"
        @Pipe({
            name: 'test',
            standalone: true
        })
        export class TestPipe {}
        "#;
        let tree = analyzer.parse_file(standalone_content)?;
        assert!(analyzer.extract_pipe_standalone_flag(tree.root_node(), standalone_content.as_bytes()));
        
        // Test standalone: false
        let non_standalone_content = r#"
        @Pipe({
            name: 'test',
            standalone: false
        })
        export class TestPipe {}
        "#;
        let tree = analyzer.parse_file(non_standalone_content)?;
        assert!(!analyzer.extract_pipe_standalone_flag(tree.root_node(), non_standalone_content.as_bytes()));
        
        // Test default (should be false)
        let default_content = r#"
        @Pipe({
            name: 'test'
        })
        export class TestPipe {}
        "#;
        let tree = analyzer.parse_file(default_content)?;
        assert!(!analyzer.extract_pipe_standalone_flag(tree.root_node(), default_content.as_bytes()));
        
        Ok(())
    }
}