use std::{fs::File, io::{BufReader, Read}, path::Path};

pub fn damn<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let _raymarch = File::open(path).unwrap();
    let mut bytes = Vec::<u8>::new();
    BufReader::new(_raymarch).read_to_end(&mut bytes).unwrap();
    bytes
}

#[macro_export]
macro_rules! asset {
    ($file:expr, $assets:expr) => {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                {
                    /*
                    with_builtin_macros::with_builtin!(let $bytes = include_bytes_from_root!(env!($file)) in {
                        $assets.insert($file, $bytes.to_vec());
                        //$assets.import(path, $bytes.to_vec());
                    });
                    */

                    $assets.insert($file, damn(env!($file)));
                    println!("Loading asset {} dynamically at runtime...", $file);
                    println!(env!($file));
                }
            } else {
                let bytes = include_bytes!(env!($file));
                $assets.insert($file, bytes.to_vec());
                println!("Loading embedded asset {} from compile time...", $file);
                
                //let path = concat!(env!("CARGO_MANIFEST_DIR"), $prefix, $file);
                //assets.hijack($file, path);
            }
        }
        /*
        
                {
                    with_builtin_macros::with_builtin!(let $bytes = include_bytes_from_root!(concat!(
                        env!("CARGO_MANIFEST_DIR"), "/", $prefix, $file,
                    )) in {
                        let path = $file;
                        dbg!(&path);
                        $assets.import(path, $bytes.to_vec());
                    });
                }
            } else {
                let path = concat!(env!("CARGO_MANIFEST_DIR"), $prefix, $file);
                $assets.hijack($file, path);
            }
        }
        */
    };
}