use std::{
    fs::{self, File, Metadata},
    process,
    time::SystemTime,
};

use chrono::NaiveDateTime;
use clap::{load_yaml, App, AppSettings::ColoredHelp, ArgMatches};
use filetime::{set_file_atime, set_file_mtime, set_file_times, set_symlink_file_times, FileTime};

// TODO: add Unit tests for touch
#[cfg(test)]
mod tests;

fn main() {
    let yaml = load_yaml!("touch.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    let flags = TouchFlags::from_matches(&matches);

    // get files list in argument
    // Required argument, ok to unwrap and not check if is supplied.
    let files: Vec<_> = matches.values_of("FILE").unwrap().collect();

    touch(&files, flags);
}

#[derive(Debug, Clone, Copy)]
struct TouchFlags<'a> {
    access_time: bool,
    mod_time: bool,
    no_create: bool,
    no_deref: bool,
    date: bool,
    date_val: &'a str,
}

impl<'a> TouchFlags<'a> {
    fn from_matches(matches: &'a ArgMatches<'a>) -> Self {
        let time_val = matches.value_of("time").unwrap_or("");
        let mut access_time = matches.is_present("accesstime")
            || time_val == "access"
            || time_val == "atime"
            || time_val == "use";

        let mut mod_time =
            matches.is_present("modification") || time_val == "modify" || time_val == "mtime";

        if !access_time && !mod_time {
            access_time = true;
            mod_time = true;
        }

        TouchFlags {
            access_time,
            mod_time,
            no_create: matches.is_present("nocreate") || matches.is_present("no_deref"),
            no_deref: matches.is_present("no_deref"),
            date: matches.is_present("date"),
            date_val: matches.value_of("date").unwrap_or(""),
        }
    }
}

fn touch(files: &[&str], flags: TouchFlags) {
    for filename in files {
        // if file already exist in the current directory
        let file_metadata =
            if flags.no_deref { fs::symlink_metadata(&filename) } else { fs::metadata(&filename) };

        if file_metadata.is_err() && !flags.no_create {
            match File::create(&filename) {
                Ok(_) => (),
                Err(e) => eprintln!("touch: Failed to create file {}: {}", &filename, e),
            }
        } else if flags.date {
            let native_date = NaiveDateTime::parse_from_str(&flags.date_val, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_else(|err| {
                    // If there is problems parsing the
                    eprintln!("touch: Problem parsing date arguments: {}", err);
                    process::exit(1);
                });
            let newfile_time = FileTime::from_unix_time(
                native_date.timestamp(),
                native_date.timestamp_subsec_millis(),
            );

            // Ok to unwrap cause it was checked in the first condition of the if-elseif-else
            // expression.
            update_time(&filename, newfile_time, &file_metadata.unwrap(), flags);
        } else {
            let newfile_time = FileTime::from_system_time(SystemTime::now());

            // Ok to unwrap cause it was checked in the first condition of the if-elseif-else
            // expression.
            update_time(&filename, newfile_time, &file_metadata.unwrap(), flags);
        }
    }
}

fn update_time(path: &str, new_filetime: FileTime, meta: &Metadata, flags: TouchFlags) {
    match (flags.access_time, flags.mod_time) {
        (true, false) => update_access_time(&path, new_filetime, meta, flags.no_deref),
        (false, true) => update_modification_time(&path, new_filetime, meta, flags.no_deref),
        (true, true) => update_both_time(&path, new_filetime, flags.no_deref),

        // Unreachable because when creating `TouchFlags` if both are false, we change both to true
        // since de default behaviour is to change both. So (false, false) will never happen, and if
        // happen, it's a bug.
        _ => unreachable!(),
    }
}

fn update_access_time(path: &str, filetime: FileTime, meta: &Metadata, no_deref: bool) {
    if no_deref {
        let mtime = FileTime::from_last_modification_time(meta);

        if let Err(err) = set_symlink_file_times(&path, filetime, mtime) {
            eprintln!("touch: Failed to update {} access time: {}", &path, err);
        }
    } else if let Err(err) = set_file_atime(&path, filetime) {
        eprintln!("touch: Failed to update {} access time: {}", &path, err);
    }
}

fn update_modification_time(path: &str, filetime: FileTime, meta: &Metadata, no_deref: bool) {
    if no_deref {
        let atime = FileTime::from_last_access_time(meta);

        if let Err(err) = set_symlink_file_times(&path, atime, filetime) {
            eprintln!("touch: Failed to update {} modification time: {}", &path, err);
        }
    } else if let Err(err) = set_file_mtime(&path, filetime) {
        eprintln!("touch: Failed to update {} modification time: {}", &path, err);
    }
}

fn update_both_time(path: &str, filetime: FileTime, no_deref: bool) {
    if no_deref {
        if let Err(err) = set_symlink_file_times(&path, filetime, filetime) {
            eprintln!("touch: Failed to update {} time: {}", &path, err);
        }
    } else if let Err(err) = set_file_times(&path, filetime, filetime) {
        eprintln!("touch: Failed to update {} time: {}", &path, err);
    }
}
