// ! CLI help text functions

/// Print export command help
pub fn print_export_help() {
    println!("Export profiles to encrypted file");
    println!();
    println!("USAGE:");
    println!("    slack-rs auth export [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --profile <name>           Export specific profile (default: 'default')");
    println!("    --all                      Export all profiles");
    println!("    --out <file>               Output file path (required)");
    println!("    --passphrase-env <var>     Environment variable containing passphrase");
    println!("    --passphrase-prompt        Prompt for passphrase");
    println!("    --yes                      Confirm dangerous operation (required)");
    println!("    --lang <code>              Language code (en/ja)");
    println!("    -h, --help                 Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Export default profile");
    println!("    export PASSPHRASE=mysecret");
    println!("    slack-rs auth export --out backup.enc --passphrase-env PASSPHRASE --yes");
    println!();
    println!("    # Export all profiles with prompt");
    println!("    slack-rs auth export --all --out all-profiles.enc --passphrase-prompt --yes");
}

/// Print import command help
pub fn print_import_help() {
    println!("Import profiles from encrypted file");
    println!();
    println!("USAGE:");
    println!("    slack-rs auth import [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --in <file>                Input file path (required)");
    println!("    --passphrase-env <var>     Environment variable containing passphrase");
    println!("    --passphrase-prompt        Prompt for passphrase");
    println!("    --yes                      Automatically accept conflicts");
    println!("    --force                    Overwrite existing profiles");
    println!("    --dry-run                  Preview changes without writing");
    println!("    --json                     Output import result as JSON");
    println!("    --lang <code>              Language code (en/ja)");
    println!("    -h, --help                 Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Import from encrypted file");
    println!("    export PASSPHRASE=mysecret");
    println!("    slack-rs auth import --in backup.enc --passphrase-env PASSPHRASE");
    println!();
    println!("    # Import with force overwrite");
    println!("    slack-rs auth import --in backup.enc --passphrase-prompt --force --yes");
    println!();
    println!("    # Preview import plan as JSON without writing changes");
    println!(
        "    slack-rs auth import --in backup.enc --passphrase-env PASSPHRASE --dry-run --json"
    );
}
