use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings::ColoredHelp, Arg,
};

pub(crate) fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .help_message("Display help information.")
        .version_message("Display version information.")
        .help_short("?")
        .settings(&[ColoredHelp])
        .arg(Arg::with_name("COMMAND").help("Command or utility to run.").required(true))
        .arg(
            Arg::with_name("ARGUMENT")
                .help("Optional arguments to pass to <COMMAND>.")
                .multiple(true),
        )
        .arg(
            Arg::with_name("posix")
                .help(
                    "Display time output in POSIX specified format as:\n\treal %f\n\tuser \
                     %f\n\tsys  %f\nTimer accuracy is arbitrary, but will always be counted in \
                     seconds.",
                )
                .long("posix")
                .short("p")
                .takes_value(false),
        )
}
