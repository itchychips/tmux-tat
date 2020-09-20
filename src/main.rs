use std::ffi::OsString;
use std::fs::DirEntry;
use std::io::{self, Read};
use std::os::unix::fs::FileTypeExt;
use std::path::PathBuf;
use std::process::Command;

use libc::getuid;

use os_pipe::pipe;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program_name = args.get(0).unwrap();
    let program_name = PathBuf::from(program_name);
    let program_name = program_name.file_name().unwrap();
    let _arg1 = args.get(1);
    let _arg2 = args.get(2);

    if program_name == "tat" {
        todo!();
    }
    else {
        if program_name != "tls" {
            eprintln!("Program name should be tls.  Is: {:?}.  Continuing with tls.", program_name);
        }
        // Feature parity with bash would be not to check these, but we'll leave it in for a future
        // addition.
        //if _arg1.is_some() || _arg2.is_some() {
        //    eprintln!("tls does not take any arguments.");
        //    std::process::exit(1);
        //}
        tmux_list_session();
    }
}

fn tmux_list_session() {
    // Not sure if tmux uses uid or euid.
    let user_id = unsafe { getuid() };

    let tmux_temporary_directory = std::env::var_os("TMUX_TMPDIR")
        .unwrap_or(OsString::from("/tmp"));
    let tmux_temporary_directory = PathBuf::from(tmux_temporary_directory);
    let tmux_socket_directory = tmux_temporary_directory
        .join(PathBuf::from(format!("tmux-{}", user_id)));

    // Sort for feature parity of the sort of the bash script.
    let mut entries: Vec<io::Result<DirEntry>> = std::fs::read_dir(tmux_socket_directory).unwrap().collect();
    entries.sort_by_key(|x| x.as_ref().unwrap().file_name() );

    // Now actually list sessions on each entry if it is a socket.
    for entry in entries {
        let entry = entry.unwrap();
        let is_socket = match entry.file_type() {
            Ok(f) => f.is_socket(),
            _ => false,
        };
        if !is_socket {
            continue;
        }

        let entry_path = entry.path();
        match entry.file_name().into_string() {
            Ok(s) => println!("Socket {}:", s),
            Err(e) => {
                eprintln!("Socket name may not be valid utf-8: {:?}", e);
                println!("Socket {:?}:", entry.file_name());
            },
        }

        // Join stdout and stderr in the child process.
        let (mut reader, writer) = pipe().unwrap();
        let writer2 = writer.try_clone().unwrap();
        let _output = Command::new("tmux")
            .stdout(writer)
            .stderr(writer2)
            .arg("-S")
            .arg(entry_path)
            .arg("list-session")
            .output()
            .expect("whoa");

        let mut s = String::new();
        reader.read_to_string(&mut s).unwrap();
        for line in s.lines() {
            println!("    {}", line);
        }
    }
}
