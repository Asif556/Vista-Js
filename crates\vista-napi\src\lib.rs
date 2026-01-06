use napi_derive::napi;
use vista_transforms::{detect_client_directive_fast, has_client_directive};
use std::path::Path;
use std::fs;

#[napi]
pub fn is_client_component(source: String) -> bool {
    has_client_directive(&source)
}

#[napi]
pub fn analyze_client_directive(source: String) -> ClientDirectiveInfo {
    let result = detect_client_directive_fast(&source);
    ClientDirectiveInfo {
        is_client: result.is_client,
        directive_line: result.directive_line as u32,
    }
}

#[napi(object)]
pub struct ClientDirectiveInfo {
    pub is_client: bool,
    pub directive_line: u32,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct RouteNode {
    pub segment: String,
    pub kind: String, 
    pub index_path: Option<String>, 
    pub layout_path: Option<String>, 
    pub loading_path: Option<String>, 
    pub error_path: Option<String>, 
    pub not_found_path: Option<String>, 
    pub children: Vec<RouteNode>,
}

fn get_file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "".to_string())
}

fn get_route_kind(segment: &str) -> String {
    if segment.starts_with('(') && segment.ends_with(')') {
        "group".to_string()
    } else if segment.starts_with('[') && segment.ends_with(']') {
        if segment.starts_with("[...") {
            "catch-all".to_string()
        } else {
            "dynamic".to_string()
        }
    } else {
        "static".to_string()
    }
}

fn build_route_node(dir_path: &Path, base_path: &Path) -> RouteNode {
    let dir_name = get_file_name(dir_path);
    let segment = match dir_path.file_name() {
        Some(file_name) => {
            let file_name_str = file_name.to_string_lossy();
            if dir_path == base_path {
                "".to_string()
            } else {
                file_name_str.to_string()
            }
        }
        None => "".to_string(),
    };

    let kind = get_route_kind(&segment);

    let mut node = RouteNode {
        segment,
        kind,
        index_path: None,
        layout_path: None,
        loading_path: None,
        error_path: None,
        not_found_path: None,
        children: Vec::new(),
    };

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = get_file_name(&path);

            if path.is_dir() {
                if !file_name.starts_with('.') && file_name != "node_modules" {
                    let child_node = build_route_node(&path, base_path);
                    if child_node.index_path.is_some() 
                        || child_node.layout_path.is_some() 
                        || !child_node.children.is_empty() {
                        node.children.push(child_node);
                    }
                }
            } else {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let full_path = path.to_string_lossy().to_string();

                if !(file_name.ends_with(".tsx") 
                    || file_name.ends_with(".ts") 
                    || file_name.ends_with(".jsx") 
                    || file_name.ends_with(".js")) {
                    continue;
                }

                match stem {
                    "page" | "index" => node.index_path = Some(full_path),
                    "layout" | "root" => node.layout_path = Some(full_path),
                    "loading" => node.loading_path = Some(full_path),
                    "error" => node.error_path = Some(full_path),
                    "not-found" => node.not_found_path = Some(full_path),
                    _ => {}
                }
            }
        }
    }

    node.children.sort_by(|a, b| {
        let order_a = match a.kind.as_str() { "static" => 0, "dynamic" => 1, _ => 2 };
        let order_b = match b.kind.as_str() { "static" => 0, "dynamic" => 1, _ => 2 };
        if order_a != order_b {
            return order_a.cmp(&order_b);
        }
        a.segment.cmp(&b.segment)
    });

    node
}

#[napi]
pub fn get_route_tree(app_dir: String) -> RouteNode {
    let root_path = Path::new(&app_dir);
    build_route_node(root_path, root_path)
}

#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[napi]
pub fn has_metadata_export(source: String) -> bool {
    source.contains("export const metadata") || source.contains("export let metadata")
}

#[napi]
pub fn has_generate_metadata(source: String) -> bool {
    source.contains("export function generateMetadata") 
    || source.contains("export async function generateMetadata") 
    || source.contains("export const generateMetadata")
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct MetadataInfo {
    pub has_static_metadata: bool,
    pub has_generate_metadata: bool,
}

#[napi]
pub fn analyze_metadata(source: String) -> MetadataInfo {
    MetadataInfo {
        has_static_metadata: has_metadata_export(source.clone()),
        has_generate_metadata: has_generate_metadata(source),
    }
}

fn convert_component(c: &vista_transforms::rsc::ScannedComponent) -> NapiScannedComponent {
    NapiScannedComponent {
        absolute_path: c.absolute_path.clone(),
        relative_path: c.relative_path.clone(),
        is_client: c.is_client,
        directive_line: c.directive_line as u32,
        component_type: format!("{:?}", c.component_type).to_lowercase(),
        exports: c.exports.clone(),
        client_hooks_used: c.client_hooks_used.clone(),
        has_metadata: c.has_metadata,
        has_generate_metadata: c.has_generate_metadata,
    }
}

#[napi]
pub fn rsc_scan_app(app_dir: String) -> NapiScanResult {
    let result = vista_transforms::rsc::scan_app_directory(&app_dir);
    
    NapiScanResult {
        client_components: result.client_components.iter().map(convert_component).collect(),
        server_components: result.server_components.iter().map(convert_component).collect(),
        pages: result.pages.iter().map(convert_component).collect(),
        layouts: result.layouts.iter().map(convert_component).collect(),
        api_routes: result.api_routes.iter().map(convert_component).collect(),
        errors: result.errors.iter().map(|e| NapiServerComponentError {
            file: e.file.clone(),
            message: e.message.clone(),
            hooks: e.hooks.clone(),
        }).collect(),
        total_files: result.total_files as u32,
        scan_time_ms: result.scan_time_ms as u32,
    }
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiScannedComponent {
    pub absolute_path: String,
    pub relative_path: String,
    pub is_client: bool,
    pub directive_line: u32,
    pub component_type: String,
    pub exports: Vec<String>,
    pub client_hooks_used: Vec<String>,
    pub has_metadata: bool,
    pub has_generate_metadata: bool,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiServerComponentError {
    pub file: String,
    pub message: String,
    pub hooks: Vec<String>,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiScanResult {
    pub client_components: Vec<NapiScannedComponent>,
    pub server_components: Vec<NapiScannedComponent>,
    pub pages: Vec<NapiScannedComponent>,
    pub layouts: Vec<NapiScannedComponent>,
    pub api_routes: Vec<NapiScannedComponent>,
    pub errors: Vec<NapiServerComponentError>,
    pub total_files: u32,
    pub scan_time_ms: u32,
}

#[napi]
pub fn rsc_generate_client_manifest(app_dir: String, build_id: String) -> NapiClientManifest {
    let manifest = vista_transforms::rsc::generate_client_manifest(&app_dir, &build_id);
    
    NapiClientManifest {
        build_id: manifest.build_id,
        client_modules: manifest.client_modules.values().map(|e| NapiClientModuleEntry {
            id: e.id.clone(),
            path: e.path.clone(),
            absolute_path: e.absolute_path.clone(),
            chunk_name: e.chunk_name.clone(),
            exports: e.exports.clone(),
            async_load: e.async_load,
        }).collect(),
    }
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiClientModuleEntry {
    pub id: String,
    pub path: String,
    pub absolute_path: String,
    pub chunk_name: String,
    pub exports: Vec<String>,
    pub async_load: bool,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiClientManifest {
    pub build_id: String,
    pub client_modules: Vec<NapiClientModuleEntry>,
}

#[napi]
pub fn rsc_generate_server_manifest(app_dir: String, build_id: String) -> NapiServerManifest {
    let manifest = vista_transforms::rsc::generate_server_manifest(&app_dir, &build_id);
    
    NapiServerManifest {
        build_id: manifest.build_id,
        server_modules: manifest.server_modules.values().map(|e| NapiServerModuleEntry {
            id: e.id.clone(),
            path: e.path.clone(),
            absolute_path: e.absolute_path.clone(),
            component_type: e.component_type.clone(),
            has_metadata: e.has_metadata,
            has_generate_metadata: e.has_generate_metadata,
        }).collect(),
        routes: manifest.routes.iter().map(|r| NapiRouteEntry {
            pattern: r.pattern.clone(),
            page_path: r.page_path.clone(),
            layout_paths: r.layout_paths.clone(),
            loading_path: r.loading_path.clone(),
            error_path: r.error_path.clone(),
            route_type: r.route_type.clone(),
        }).collect(),
    }
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiRouteEntry {
    pub pattern: String,
    pub page_path: String,
    pub layout_paths: Vec<String>,
    pub loading_path: Option<String>,
    pub error_path: Option<String>,
    pub route_type: String,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiServerModuleEntry {
    pub id: String,
    pub path: String,
    pub absolute_path: String,
    pub component_type: String,
    pub has_metadata: bool,
    pub has_generate_metadata: bool,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiServerManifest {
    pub build_id: String,
    pub server_modules: Vec<NapiServerModuleEntry>,
    pub routes: Vec<NapiRouteEntry>,
}

#[napi]
pub fn rsc_generate_mount_id() -> String {
    vista_transforms::rsc::generate_mount_id()
}

#[napi]
pub fn rsc_reset_mount_counter() {
    vista_transforms::rsc::reset_mount_counter()
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiPrerenderedComponent {
    pub component_id: String,
    pub placeholder_html: String,
    pub estimated_height: u32,
}

#[napi]
pub fn rsc_prerender_component(file_path: String) -> Option<NapiPrerenderedComponent> {
    vista_transforms::rsc::prerender_client_component(&file_path).map(|c| NapiPrerenderedComponent {
        component_id: c.component_id,
        placeholder_html: c.placeholder_html,
        estimated_height: c.estimated_height,
    })
}

#[napi]
pub fn rsc_prerender_all_components(app_dir: String) -> std::collections::HashMap<String, NapiPrerenderedComponent> {
    vista_transforms::rsc::prerender_all_client_components(&app_dir)
        .into_iter()
        .map(|(k, v)| (k, NapiPrerenderedComponent {
            component_id: v.component_id,
            placeholder_html: v.placeholder_html,
            estimated_height: v.estimated_height,
        }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_client() {
        assert!(is_client_component("'client load';\n".to_string()));
        assert!(!is_client_component("export default function() {}".to_string()));
    }
}