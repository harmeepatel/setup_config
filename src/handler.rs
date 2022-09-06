use ansi_term::Color;
use std::io::{self, Write};

#[warn(unused_must_use)]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Symlink {
    pub from: String,
    pub to: String,
}

enum Remove {
    Yes,
    No,
}

impl Symlink {
    fn new() -> Symlink {
        Symlink {
            to: String::new(),
            from: String::new(),
        }
    }
}

type Symlinks = Vec<Symlink>;

fn replace_home(arr: &mut Symlinks, remove: Remove) -> Symlinks {
    arr.iter()
        .map(|sym| {
            let env_home = match std::env::var("HOME") {
                Ok(home_user) => home_user,
                Err(_) => String::from("Set $HOME."),
            };

            let env_house = match remove {
                Remove::Yes => vec![&env_home as &str],
                Remove::No => vec!["$HOME", "~"],
            };

            let home = match remove {
                Remove::Yes => "~",
                Remove::No => &env_home,
            };
            let mut temp = Symlink::new();

            for i in env_house {
                if sym.to.contains(i) || sym.from.contains(i) {
                    temp.to = sym.to.replace(i, &home);
                    temp.from = sym.from.replace(i, &home);
                }
            }
            temp
        })
        .collect()
}

fn remove_non_existing(arr: &mut Symlinks, idx_remove: &Vec<usize>, config_path: &String) {
    print!(
        "{}",
        Color::White
            .italic()
            .paint("Want to remove following objects from json config? (y/n): ")
    );
    io::stdout().flush().unwrap();

    let mut rem_reply = String::new();
    std::io::stdin().read_line(&mut rem_reply).unwrap();
    let rem_reply = rem_reply.trim_end();

    if rem_reply == "y" {
        // dbg!(&arr, &path);
        let mut c = 0;
        for i in idx_remove {
            let shifted_index = i - c;
            let _ = arr.remove(shifted_index);
            c += 1;
        }
        let arr: Symlinks = replace_home(arr, Remove::Yes);

        match &std::fs::File::create(config_path) {
            Ok(file) => match serde_json::to_writer(file, &arr) {
                Ok(()) => {
                    println!("{}", Color::Blue.italic().paint("Removed."))
                }
                Err(err) => {
                    println!(
                        "{}\n{}",
                        Color::Red
                            .bold()
                            .paint("Something went wrong in writing the file!!!"),
                        err
                    )
                }
            },
            Err(err) => {
                println!(
                    "{}\n{}",
                    Color::Red
                        .bold()
                        .paint("Something went wrong in creating the file!!!"),
                    err
                );
            }
        }
    } else {
        println!("{}", Color::White.italic().paint("Bye👋."));
    }
}

pub fn create_syms(buf: &String, path: &String, arg_remove: &bool) {
    let mut exist_sym_count = 0;
    let mut to_remove: Vec<usize> = Vec::new();

    let mut arr: Symlinks = serde_json::from_str(buf).unwrap();
    let mut arr: Symlinks = replace_home(&mut arr, Remove::No);

    for (id, sym) in arr.clone().iter().enumerate() {
        let to_meta = match std::fs::symlink_metadata(&sym.to) {
            Ok(metadata) => Ok(metadata),
            Err(_) => Err(()),
        };

        let from_meta = match std::fs::symlink_metadata(&sym.from) {
            Ok(metadata) => Ok(metadata),
            Err(_) => Err(()),
        };

        if from_meta.is_ok() {
            if let Ok(meta) = to_meta {
                if meta.file_type().is_symlink() {
                    exist_sym_count += 1;
                }
            } else {
                println!("Creating link from: \t{}\n\t\tto: \t{}", sym.from, sym.to);
                std::os::unix::fs::symlink(&sym.from, &sym.to).unwrap();
            }
        } else {
            println!(
                "{}{}",
                Color::Red.bold().paint("404: "),
                Color::Red.bold().paint(&sym.from)
            );
            to_remove.push(id);
        }
    }

    // dbg!(&to_remove);
    if to_remove.len() > 0 && *arg_remove {
        remove_non_existing(&mut arr, &to_remove, &path);
    }

    if exist_sym_count == arr.len() && !*arg_remove {
        println!("{}", Color::Blue.italic().paint("Everything is fine!!"));
    }
}

pub fn write(to: &String, from: &String) {
    dbg!(&to, &from);
}
