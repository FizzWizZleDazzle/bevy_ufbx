//! Loads every FBX file from the ufbx test suite and verifies our processing
//! functions handle them without panicking or returning unexpected errors.
//!
//! Set the `UFBX_TEST_DATA` environment variable to the path of the
//! `ufbx/data` directory before running:
//!
//! ```sh
//! UFBX_TEST_DATA=/tmp/ufbx/data cargo test load_all_ufbx_test_data -- --nocapture
//! ```

use bevy_ufbx::material::create_standard_material;
use bevy_ufbx::mesh::group_faces_by_material;
use bevy_ufbx::utils::{convert_matrix, convert_transform};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Subdirectories that contain binary fuzz/garbage data, not valid FBX files.
const SKIP_DIRS: &[&str] = &["fuzz", "cache_fuzz", "obj_fuzz", "mtl_fuzz"];

fn collect_fbx_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_recursive(root, &mut files);
    files.sort();
    files
}

fn collect_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if !SKIP_DIRS.contains(&name) {
                collect_recursive(&path, out);
            }
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("fbx"))
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
}

/// Attempt to load an FBX file through the full processing pipeline used by
/// our loader, without needing a `LoadContext`.
///
/// Returns `Ok(())` on success or a human-readable error string.
fn try_process_fbx(path: &Path) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read: {e}"))?;

    let root = ufbx::load_memory(
        &bytes,
        ufbx::LoadOpts {
            target_unit_meters: 1.0,
            target_axes: ufbx::CoordinateAxes::right_handed_y_up(),
            ..Default::default()
        },
    )
    .map_err(|e| format!("ufbx: {e:?}"))?;

    let scene: &ufbx::Scene = &*root;

    // --- Nodes / transforms ---
    for node in scene.nodes.as_ref().iter() {
        convert_transform(&node.local_transform);
        convert_matrix(&node.node_to_world);

        // Mesh face grouping (pure data, no LoadContext needed)
        if let Some(mesh_ref) = node.mesh.as_ref() {
            let mesh = mesh_ref.as_ref();
            if mesh.num_vertices > 0 && !mesh.faces.as_ref().is_empty() {
                group_faces_by_material(mesh);
            }
        }
    }

    // --- Materials ---
    // Pass an empty texture map; we're testing structural correctness, not
    // texture path resolution.
    let no_textures = HashMap::new();
    for mat in scene.materials.as_ref().iter() {
        if mat.element.element_id == 0 {
            continue;
        }
        create_standard_material(mat, &no_textures)
            .map_err(|e| format!("material '{}': {e:?}", mat.element.name))?;
    }

    Ok(())
}

#[test]
fn load_all_ufbx_test_data() {
    let data_dir = match std::env::var("UFBX_TEST_DATA") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            println!("Skipping: UFBX_TEST_DATA env var not set.");
            println!("To run: UFBX_TEST_DATA=/path/to/ufbx/data cargo test load_all_ufbx_test_data -- --nocapture");
            return;
        }
    };

    assert!(
        data_dir.is_dir(),
        "UFBX_TEST_DATA '{}' is not a directory",
        data_dir.display()
    );

    let fbx_files = collect_fbx_files(&data_dir);
    assert!(
        !fbx_files.is_empty(),
        "No .fbx files found in '{}'",
        data_dir.display()
    );

    println!("Found {} FBX files in {}\n", fbx_files.len(), data_dir.display());

    let mut passed = 0usize;
    let mut failures: Vec<String> = Vec::new();
    let total_start = Instant::now();

    for path in &fbx_files {
        let rel = path.strip_prefix(&data_dir).unwrap_or(path);
        let t = Instant::now();
        match try_process_fbx(path) {
            Ok(()) => {
                passed += 1;
                println!("  OK  {:>6}ms  {}", t.elapsed().as_millis(), rel.display());
            }
            Err(e) => {
                let msg = format!("{} — {}", rel.display(), e);
                eprintln!("FAIL  {:>6}ms  {}", t.elapsed().as_millis(), msg);
                failures.push(msg);
            }
        }
    }

    let total = fbx_files.len();
    println!(
        "\n{passed}/{total} passed in {:.2}s",
        total_start.elapsed().as_secs_f64()
    );

    if !failures.is_empty() {
        panic!(
            "\n{} file(s) failed to process:\n  {}",
            failures.len(),
            failures.join("\n  ")
        );
    }
}
