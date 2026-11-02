use std::process::Command;

#[test]
fn help_and_invalid_arguments_are_handled_by_the_parser() {
    let executable = env!("CARGO_BIN_EXE_tilbud");
    let help = Command::new(executable).arg("--help").output().unwrap();
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(text.contains("tilbud"));
    assert!(text.contains("--help"));
    let invalid = Command::new(executable).arg("--ukendt").output().unwrap();
    assert!(!invalid.status.success());
}
