use std::collections::HashMap;

fn main() {
    let file = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let parse_map = parse(&file);
    let mut pkg = Package::default();
    expr_to_pkg(parse_map, &mut pkg);
    pkg.install_pkg();
}

fn parse(file: &str) -> HashMap<String, Vec<String>> {
    let mut parse_map: HashMap<String, Vec<String>> = HashMap::new();
    // multiline entry flag
    let mut current_lexpr = None;

    // unesacpe escapes
    let file = file.replace("\\\n", " ");

    let is_not_comment = |l: &&str| -> bool { !l.trim_start().starts_with('#') };
    for line in file.lines().filter(is_not_comment) {
        // if line starts with a tab then its an rexpr
        // it should be added to the last found lexpr's values
        if line.starts_with('\t') || line.starts_with("    ") {
            parse_map
                .get_mut(&current_lexpr.clone().unwrap())
                .unwrap()
                .push(line.trim().to_string());
            continue;
        }

        let mut l = line.split(':');
        // we found ':' we are at the beging of a part
        if let Some(lexpr) = l.next() {
            // trim to get rid of alot of edge cases
            let lexpr = lexpr.trim();
            // since we are at the beingin reset the multiline flag
            current_lexpr = None;
            if let Some(rexpr) = l.next() {
                // trim to get rid of alot of edge cases
                let rexpr = rexpr.trim();
                // else save the lexpr
                // and prepare to parse the next lines as its values
                // using current_lexpr as a flag
                if rexpr.is_empty() || rexpr == "|" {
                    parse_map.insert(lexpr.to_string(), vec![]);
                    current_lexpr = Some(lexpr.to_string());
                }
                // single line instruction
                else {
                    parse_map.insert(lexpr.to_string(), vec![rexpr.to_string()]);
                }
            }
        }
    }

    parse_map
}

fn expr_to_pkg(parse_map: std::collections::HashMap<String, Vec<String>>, pkg: &mut Package) {
    for (lexpr, rexpr) in parse_map {
        match lexpr.trim() {
            "name" => {
                pkg.name = Some(rexpr.fst());
            }
            "version" => {
                pkg.version = Some(rexpr.fst());
            }
            "release" => {
                pkg.release = Some(rexpr.fst().parse().unwrap());
            }
            "source" => {
                pkg.source = Some(rexpr);
            }
            "license" => {
                pkg.license = Some(rexpr);
            }
            "component" => {
                pkg.component = Some(rexpr.fst());
            }
            "summary" => {
                pkg.summary = Some(rexpr.fst());
            }
            "description" => {
                pkg.description = Some(rexpr.fst());
            }
            "builddeps" => {
                pkg.builddeps = Some(rexpr);
            }
            "setup" => {
                pkg.setup = Some(rexpr);
            }
            "build" => {
                pkg.build = Some(rexpr);
            }
            "install" => {
                pkg.install = Some(rexpr);
            }
            x => eprintln!("Uknown lexpr: {}", x),
        }
    }
}

#[derive(Debug, Default)]
struct Package {
    name: Option<String>,
    version: Option<String>,
    release: Option<String>,
    source: Option<Vec<String>>,
    license: Option<Vec<String>>,
    component: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    builddeps: Option<Vec<String>>,
    setup: Option<Vec<String>>,
    build: Option<Vec<String>>,
    install: Option<Vec<String>>,
    cmds: HashMap<String, String>,
}
use std::process::Command;
impl Package {
    fn install_pkg(&mut self) {
        self.get_cmds();
        self.download_source();
        self.install_builddeps();
        self.setup();
        self.build();
        self.install();
    }

    fn get_cmds(&mut self) {
        let config = std::fs::read_to_string("solar_cmds").unwrap();
        config.lines().for_each(|l| {
            let mut l = l.split(':');
            self.cmds.insert(
                l.next().unwrap().trim().into(),
                l.next().unwrap().trim().into(),
            );
        });
    }

    fn download_source(&self) {
    	//TODO
    } 

    fn install_builddeps(&self) {
        let install_bd_cmd = &self.cmds["install_builddep"];
        let builddeps: Vec<String> = self
            .builddeps
            .clone()
            .unwrap()
            .iter()
            .map(|d: &String| {
                let mut dep = d.split('-').nth(1).unwrap().trim().to_string();
                // special case: pkgconfig
                if dep.contains("pkgconfig") {
                    dep = dep.split("pkgconfig(").nth(1).unwrap().to_string();
                    // pop )
                    dep.pop();
                }
                dep
            })
            .collect();

        //log
        println!("installing build dependencies: {:?}", &builddeps);

        for dep in builddeps {
            let cmd = install_bd_cmd.clone() + " " + &dep;

            Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }
    }

    fn setup(&self) {
        run_solus_cmds(self.setup.clone().unwrap());
    }

    fn build(&self) {
        run_solus_cmds(self.build.clone().unwrap());
    }

    fn install(&self) {
        run_solus_cmds(self.install.clone().unwrap());
    }
}

fn run_solus_cmds(cmds: Vec<String>) {
    let cmds: Vec<String> = cmds
        .into_iter()
        .map(|mut s| {
            // cmds starts with '%'
            if s.starts_with('%') {
                s.remove(0);
            }
            // %reconfigure is alais for ./configure
            if s.starts_with("reconfigure") {
                s = s.replace("reconfigure", "./configure");
            }
            s
        })
        .collect();

    for cmd in cmds {
        Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}

// ----helper methods-----

trait VecTools<T> {
    fn fst(self) -> T;
}
impl<T> VecTools<T> for Vec<T> {
    fn fst(self) -> T {
        self.into_iter().next().unwrap()
    }
}
