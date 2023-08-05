use clap::Parser;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use toml::{Table, Value};

#[derive(Debug, Parser)]
pub struct Config {
  path: PathBuf,
  out: PathBuf,
}

type XResult<T> = Result<T, Box<dyn Error>>;

pub fn run(config: Config) -> XResult<()> {
  let dependencies = tranverse_subdirs(config.path)?;
  export_dot(dependencies, &config.out)?;
  std::process::Command::new("dot")
    .args([
      "-T",
      "svg",
      "-o",
      "rustc-inter-deps.svg",
      config.out.to_str().unwrap(),
    ])
    .output()
    .unwrap();
  Ok(())
}

fn tranverse_subdirs<T>(path: T) -> XResult<HashMap<String, Vec<String>>>
where
  T: AsRef<Path>,
{
  let ignored_crates = vec![
    "rustc",
    "rustc_graphviz",
    "rustc_codegen_cranelift",
    "rustc_codegen_gcc",
    "rustc_error_messages",
    "rustc_baked_icu_data",
    "rustc_fs_util",
    "rustc_hir_pretty",
    "rustc_apfloat",
    "rustc_transmute",
    "rustc_parse_format",
    "rustc_smir",
    "rustc_llvm",
    "rustc_abi",
    "rustc_monomorphize",
    "rustc_log",
    "rustc_error_codes",
    "rustc_symbol_mangling",
    "rustc_errors",
    "rustc_attr",
    "rustc_metadata",
    "rustc_feature",
    "rustc_ast_pretty",
    "rustc_index",
    "rustc_arena",
    "rustc_driver",
    "rustc_target",
    "rustc_serialize",
    "rustc_codegen_ssa",
    "rustc_privacy",
    "rustc_codegen_llvm",
    "rustc_lint_defs",
    "rustc_data_structures",
    "rustc_span",
    "rustc_ty_utils",
    "rustc_mir_build",
    "rustc_type_ir",
    "rustc_plugin_impl",
    "rustc_session",
    "rustc_lint",
    "rustc_macros",
    "rustc_builtin_macros",
    "rustc_ast_passes",
    "rustc_mir_transform",
    "rustc_trait_selection",
  ];

  let mut res = HashMap::new();
  let path = Path::new(path.as_ref());
  for subdir in path.read_dir()? {
    if let Ok(subdir) = subdir {
      if subdir.file_type()?.is_dir() {
        for entry in subdir.path().read_dir()? {
          if let Ok(entry) = entry {
            if entry.file_name() == "Cargo.toml" {
              let crate_name = subdir.file_name().to_string_lossy().to_string();

              // ignore some crates
              if ignored_crates.contains(&crate_name.as_str()) {
                continue;
              }

              let dependencies = extract_depdendency(entry.path())?;
              res.insert(crate_name, dependencies);
            }
          }
        }
      }
    }
  }
  Ok(res)
}

fn extract_depdendency(path: impl AsRef<Path>) -> XResult<Vec<String>> {
  let table = fs::read_to_string(path)?.parse::<Table>().unwrap();
  let dependencies = table.get("dependencies");
  let mut res = vec![];
  if let Some(Value::Table(table)) = dependencies {
    let keys = table.keys();
    for key in keys {
      if key.starts_with("rustc") {
        res.push(key.to_string());
      }
    }
  }
  Ok(res)
}

struct Graph {
  nodes: Vec<String>,
  edges: Vec<(usize, usize)>,
}

fn export_dot(dependencies: HashMap<String, Vec<String>>, out: impl AsRef<Path>) -> XResult<()> {
  let mut nodes = vec![];
  let mut edges = vec![];
  for crat in dependencies.keys() {
    nodes.push(crat.to_string());
  }
  nodes.sort();
  for (crat, deps) in dependencies.iter() {
    let Ok(src_id) = nodes.binary_search(crat) else {
      continue;
    };
    for dep in deps.into_iter() {
      let Ok(dst_id) = nodes.binary_search(dep) else {
        continue;
      };
      edges.push((src_id, dst_id));
    }
  }
  let graph = Graph { nodes, edges };
  let mut file = fs::File::create(out).unwrap();
  dot::render(&graph, &mut file)?;
  Ok(())
}

type Nd = usize;
type Ed<'a> = &'a (usize, usize);

impl<'a> dot::Labeller<'a, Nd, Ed<'a>> for Graph {
  fn graph_id(&'a self) -> dot::Id<'a> {
    dot::Id::new("example2").unwrap()
  }
  fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
    dot::Id::new(format!("N{}", n)).unwrap()
  }
  fn node_label<'b>(&'b self, n: &Nd) -> dot::LabelText<'b> {
    dot::LabelText::LabelStr(std::borrow::Cow::from(&self.nodes[*n]))
  }
  fn edge_label<'b>(&'b self, _: &Ed) -> dot::LabelText<'b> {
    dot::LabelText::LabelStr("&sube;".into())
  }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed<'a>> for Graph {
  fn nodes(&self) -> dot::Nodes<'a, Nd> {
    (0..self.nodes.len()).collect()
  }
  fn edges(&'a self) -> dot::Edges<'a, Ed<'a>> {
    self.edges.iter().collect()
  }
  fn source(&self, e: &Ed) -> Nd {
    e.0
  }
  fn target(&self, e: &Ed) -> Nd {
    e.1
  }
}
