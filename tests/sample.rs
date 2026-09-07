//! The sample from the brief, end to end, compared byte for byte.

const SAMPLE_INPUT: &str = include_str!("../testdata/sample_input.txt");
const SAMPLE_OUTPUT: &str = include_str!("../testdata/sample_output.txt");

#[test]
fn sample_input_produces_the_sample_output_exactly() {
    let output = martian_robots::run(SAMPLE_INPUT).expect("the sample input is valid");
    assert_eq!(output, SAMPLE_OUTPUT);
}

#[test]
fn the_third_robot_survives_only_because_of_the_second_robots_scent() {
    // Robot 2 is lost from (3, 3) heading north. Robot 3 later reaches (3, 3)
    // heading north and tries the same move; the scent makes it a no-op and
    // the robot carries on to (2, 3) facing south.
    let without_robot_two = "5 3\n0 3 W\nLLFFFLFLFL\n";
    let output = martian_robots::run(without_robot_two).expect("valid input");
    assert_eq!(output, "3 3 N LOST\n");
}

#[test]
fn a_parse_error_reports_the_offending_line() {
    let error = martian_robots::run("5 3\n1 1 E\nRFX\n").expect_err("X is not an instruction");
    assert_eq!(error.to_string(), "line 3: unknown instruction 'X'");
}
