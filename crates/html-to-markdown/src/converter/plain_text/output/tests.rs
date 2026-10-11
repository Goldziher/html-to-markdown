//! Unit tests of the output: the places after each change, and the count of work.

use super::{Checkpoint, OpenControl, PlainOutput};

/// One change of an output.
#[derive(Clone, Copy)]
enum Step {
    Text(&'static str),
    Control(&'static str),
    Start,
    Finish,
    Options(&'static str),
    EndBlock,
    Open,
    Rollback,
    Commit,
}

/// Fails when the start of an open control is not a place in the text, or when the starts
/// do not rise.
fn assert_control_starts_are_true(output: &PlainOutput, context: &str) {
    let starts: Vec<usize> = output.open_controls.iter().map(|control| control.at).collect();
    assert!(
        starts
            .iter()
            .all(|at| *at <= output.text.whole().len() && output.text.whole().is_char_boundary(*at)),
        "{context}: the start of an open control is not a place in the text: {starts:?} in {:?}",
        output.text.whole()
    );
    assert!(
        starts.is_sorted(),
        "{context}: the starts of the open controls do not rise: {starts:?}"
    );
}

/// Fails when a control end, the start of an open control or an open checkpoint is not a
/// place in the text, or when one of the three lists does not rise.
fn assert_places_are_true(output: &PlainOutput, context: &str) {
    assert_control_starts_are_true(output, context);
    let len = output.text.whole().len();
    assert!(
        output.control_ends.as_slice().iter().all(|end| *end <= len),
        "{context}: a control end is past the text: {:?} in {:?}",
        output.control_ends.as_slice(),
        output.text.whole()
    );
    assert!(
        output.control_ends.as_slice().is_sorted(),
        "{context}: the control ends do not rise: {:?}",
        output.control_ends.as_slice()
    );
    let places: Vec<usize> = output.open_checkpoints.iter().map(|open| open.at).collect();
    assert!(
        places
            .iter()
            .all(|at| *at <= len && output.text.whole().is_char_boundary(*at)),
        "{context}: an open checkpoint is not a place in the text: {places:?} in {:?}",
        output.text.whole()
    );
    assert!(
        places.is_sorted(),
        "{context}: the open checkpoints do not rise: {places:?}"
    );
}

/// Runs `steps` on a new output, checks the places after each step and returns the text.
fn run(steps: &[Step]) -> String {
    let mut output = PlainOutput::default();
    let mut open: Vec<Checkpoint> = Vec::new();
    let mut controls: Vec<OpenControl> = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        match *step {
            Step::Text(text) => output.push_text(text),
            Step::Control(text) => {
                let control = output.start_control();
                output.push_text(text);
                output.finish_control(control);
            }
            Step::Start => controls.push(output.start_control()),
            Step::Finish => output.finish_control(controls.pop().expect("the table starts before it finishes")),
            Step::Options(options) => output.write_option_list(options, false),
            Step::EndBlock => output.end_block(),
            Step::Open => open.push(output.open()),
            Step::Rollback => output.rollback(open.pop().expect("the table opens before it closes")),
            Step::Commit => output.commit(open.pop().expect("the table opens before it closes")),
        }
        assert_places_are_true(&output, &format!("step {index}"));
    }
    output.into_text()
}

#[test]
fn a_block_end_below_one_checkpoint_keeps_the_places_true() {
    use Step::{Commit, EndBlock, Open, Rollback, Text};
    let table: [(&[Step], &str); 4] = [
        // ~keep After a return the spaces are at the end again, and the next block end removes them.
        (&[Text("aa  "), Open, EndBlock, Text("b"), Rollback, EndBlock], "aa\n\n"),
        // ~keep A block end hides spaces before a checkpoint; the return makes them text again.
        (
            &[Text("aaa   "), Open, EndBlock, Text("bcd"), EndBlock, Rollback],
            "aaa   ",
        ),
        // ~keep The same with a commit: the block end stands.
        (
            &[Text("aaa   "), Open, EndBlock, Text("bcd"), EndBlock, Commit],
            "aaa\n\nbcd\n\n",
        ),
        // ~keep A checkpoint in an empty text.
        (&[Open, EndBlock, Text("b"), EndBlock, Rollback, Text("c")], "c"),
    ];
    for (steps, expected) in table {
        assert_eq!(run(steps), expected);
    }
}

#[test]
fn a_block_end_below_two_checkpoints_keeps_the_places_true() {
    use Step::{Commit, EndBlock, Open, Rollback, Text};
    let table: [(&[Step], &str); 3] = [
        // ~keep Two checkpoints, a block end below both, the inner one kept, the outer one not.
        (
            &[
                Text("aaa "),
                Open,
                Open,
                EndBlock,
                Text("b"),
                Commit,
                Text("c"),
                Rollback,
            ],
            "aaa ",
        ),
        // ~keep Two checkpoints, a block end below both, both returned to.
        (
            &[
                Text("aaa "),
                Open,
                Text("b  "),
                Open,
                EndBlock,
                Text("c"),
                Rollback,
                Rollback,
            ],
            "aaa ",
        ),
        // ~keep The inner return makes the spaces text again; a second block end removes them again.
        (
            &[
                Text("aa  "),
                Open,
                Open,
                EndBlock,
                Text("b"),
                Rollback,
                EndBlock,
                Text("c"),
                Rollback,
            ],
            "aa  ",
        ),
    ];
    for (steps, expected) in table {
        assert_eq!(run(steps), expected);
    }
}

#[test]
fn a_control_beside_a_checkpoint_keeps_the_places_true() {
    use Step::{Commit, Control, EndBlock, Open, Options, Rollback, Text};
    let table: [(&[Step], &str); 3] = [
        // ~keep A control before the checkpoint is the last control again after the return.
        (
            &[
                Text("aaa"),
                Control("X"),
                Open,
                Control("Y"),
                Options("one two"),
                Rollback,
                Text("bbb"),
            ],
            "aaa X bbb",
        ),
        // ~keep A control that ends with spaces, a block end in a checkpoint, a return.
        (
            &[
                Text("aaa"),
                Control("X   "),
                Open,
                EndBlock,
                Text("\u{e9}b"),
                EndBlock,
                Rollback,
                Text("ccc"),
            ],
            "aaa X   ccc",
        ),
        // ~keep A control that holds a checkpoint: the space before the control moves no place wrongly.
        (
            &[Text("aaa"), Open, Control("W"), Control("Y"), Commit, Text("bbb")],
            "aaa W Y bbb",
        ),
    ];
    for (steps, expected) in table {
        assert_eq!(run(steps), expected);
    }
}

#[test]
fn a_block_end_below_the_start_of_an_open_control_keeps_the_places_true() {
    use Step::{EndBlock, Finish, Open, Rollback, Start, Text};
    let table: [(&[Step], &str); 5] = [
        // ~keep A block end in a control removes the spaces before its start: the control
        // ~keep still ends at the end of its text, and the next word gets its space.
        (
            &[Text("b   "), Start, EndBlock, Text("e"), Finish, Text("ccc")],
            "b\n\ne ccc",
        ),
        // ~keep The same with a character of two bytes: the old start was inside it.
        (
            &[Text("b   "), Start, EndBlock, Text("\u{e9}"), Finish, Text("ccc")],
            "b\n\n\u{e9} ccc",
        ),
        // ~keep A control that holds a block end and no text.
        (&[Text("b   "), Start, EndBlock, Finish, Text("ccc")], "b\n\nccc"),
        // ~keep Two open controls: the block end moves the inner start, and the space of
        // ~keep the outer control is written at its own start.
        (
            &[
                Text("a"),
                Start,
                Text("x  "),
                Start,
                EndBlock,
                Text("e"),
                Finish,
                Finish,
                Text("ccc"),
            ],
            "a x\n\ne ccc",
        ),
        // ~keep A return to a checkpoint in the control makes the spaces text again.
        (
            &[
                Text("b   "),
                Start,
                Open,
                EndBlock,
                Text("e"),
                Rollback,
                Finish,
                Text("ccc"),
            ],
            "b   ccc",
        ),
    ];
    for (steps, expected) in table {
        assert_eq!(run(steps), expected);
    }
}

#[test]
fn text_written_before_the_start_of_an_open_control_moves_the_start() {
    // ~keep No path of the writer writes text before an open control. `insert_at` keeps
    // ~keep the start true all the same: the control starts at 3, then at 5.
    let mut output = PlainOutput::default();
    output.push_text("aaa");
    let control = output.start_control();
    output.push_text("W");
    output.insert_at(1, "  ");
    assert_places_are_true(&output, "after the insert");
    output.finish_control(control);
    assert_eq!(output.text.whole(), "a  aa W");
    assert_eq!(output.control_ends.as_slice(), [7]);
    assert_eq!(output.into_text(), "a  aa W");
}

#[test]
fn an_outer_checkpoint_has_its_place_again_after_an_inner_return() {
    let mut output = PlainOutput::default();
    output.push_text("aa  ");
    let outer = output.open();
    let inner = output.open();
    output.end_block();
    output.push_text("b");
    assert_eq!(output.since(&outer), "\n\nb");
    output.rollback(inner);
    assert_eq!(output.text.whole(), "aa  ");
    assert_eq!(output.since(&outer), "");
    output.push_text("c");
    assert_eq!(output.since(&outer), "c");
    output.rollback(outer);
    assert_eq!(output.into_text(), "aa  ");
}

#[test]
fn an_outer_checkpoint_keeps_the_blanks_it_wrote_after_an_inner_return() {
    // ~keep The outer element stands after two blanks, ends a block and writes two blanks
    // ~keep of its own. A removed child with a block end leaves those in its text.
    let mut output = PlainOutput::default();
    output.push_text("a");
    output.end_block();
    output.push_text("  ");
    let outer = output.open();
    output.end_block();
    output.push_text("  ");
    let inner = output.open();
    output.end_block();
    assert_eq!(output.since(&outer), "");
    output.rollback(inner);
    assert_eq!(output.since(&outer), "  ");
    output.push_text("X");
    assert_eq!(output.since(&outer), "  X");
    output.rollback(outer);
    assert_eq!(output.into_text(), "a\n\n  ");
}

#[test]
fn a_space_written_before_a_control_moves_the_places_after_it() {
    // ~keep The outer control starts at 3 and holds a control that ends at 5 and a
    // ~keep checkpoint at 5. The space of the outer control is written at 3.
    let mut output = PlainOutput::default();
    output.push_text("aaa");
    let outer = output.start_control();
    output.push_text("W");
    let inner = output.start_control();
    output.push_text("Y");
    output.finish_control(inner);
    assert_eq!(output.text.whole(), "aaaW Y");
    assert_eq!(output.control_ends.as_slice(), [6]);
    let checkpoint = output.open();
    output.push_text("zz");
    output.finish_control(outer);
    assert_eq!(output.text.whole(), "aaa W Y zz");
    assert_eq!(output.control_ends.as_slice(), [7, 10]);
    assert_places_are_true(&output, "after the space");
    assert_eq!(output.since(&checkpoint), " zz");
    output.rollback(checkpoint);
    assert_eq!(output.text.whole(), "aaa W Y");
    assert_eq!(output.control_ends.as_slice(), [7]);
    assert_eq!(output.into_text(), "aaa W Y");
}

/// The count of control ends that the output examines for `count` controls, each after a
/// word and each with a space written before it. With `block_ends`, a block ends after
/// each control.
fn ends_examined_for(count: usize, block_ends: bool) -> usize {
    let mut output = PlainOutput::default();
    for _ in 0..count {
        output.push_text("z");
        let control = output.start_control();
        output.push_text("X");
        output.finish_control(control);
        if block_ends {
            output.end_block();
        }
    }
    assert_places_are_true(&output, "after the last control");
    assert_eq!(output.control_ends.as_slice().len(), count);
    let examined = output.control_ends.touched();
    let line = if block_ends { "z X\n\n" } else { "z X " };
    assert_eq!(output.into_text(), line.repeat(count).trim_end_matches(' '));
    examined
}

#[test]
fn controls_one_after_the_other_examine_a_bounded_count_of_control_ends() {
    // ~keep No control end stands after the place of a space, so only the binary search
    // ~keep reads ends: at most `ilog2(count) + 2` for each search, 11 and 13 here. A block
    // ~keep end makes a second search. A scan of every end reads `count * (count - 1) / 2`
    // ~keep ends for the spaces alone: 499,500 and 7,998,000.
    for (block_ends, searches) in [(false, 1_usize), (true, 2)] {
        for (count, reads) in [(1_000_usize, 11_usize), (4_000, 13)] {
            let examined = ends_examined_for(count, block_ends);
            let bound = count * reads * searches;
            assert!(
                examined <= bound,
                "{count} controls (block ends: {block_ends}) examined {examined} control ends, over {bound}"
            );
        }
    }
}

#[test]
fn removed_elements_after_a_run_of_spaces_cost_a_bounded_count_of_bytes() {
    // ~keep Each removed element holds a block end, which removes the run before it, and
    // ~keep the return makes the run text again. Each costs two line ends and three reads.
    // ~keep A read of the run or a copy of it for each element costs `count * count`.
    for count in [1_000_usize, 4_000] {
        let mut output = PlainOutput::default();
        output.push_text("b");
        output.push_text(&" ".repeat(count));
        let written = output.text.work();
        for _ in 0..count {
            let element = output.open();
            output.end_block();
            assert_eq!(output.since(&element), "\n\n");
            output.rollback(element);
        }
        let work = output.text.work() - written;
        assert!(
            work <= 8 * count,
            "{count} removed elements after {count} spaces cost {work} bytes, over 8 for each"
        );
        assert_eq!(output.len(), count + 1);
        assert_eq!(output.into_text().trim_end_matches(' '), "b");
    }
}

#[test]
fn block_ends_after_line_ends_read_a_bounded_count_of_characters() {
    // ~keep A block end reads two line ends at most. A count of all of them for each block
    // ~keep end costs `count * count`.
    for count in [1_000_usize, 4_000] {
        let mut output = PlainOutput::default();
        output.push_text("b");
        for _ in 0..count {
            output.push_mark("\n");
        }
        let written = output.text.work();
        for _ in 0..count {
            output.end_block();
        }
        let work = output.text.work() - written;
        assert!(
            work <= 4 * count,
            "{count} block ends after {count} line ends read {work} characters, over 4 for each"
        );
        assert_eq!(output.len(), count + 1);
    }
}

#[test]
fn a_commit_removes_the_hidden_spaces_and_moves_the_places_after_them() {
    let mut output = PlainOutput::default();
    output.push_text("a  ");
    let element = output.open();
    output.end_block();
    assert_eq!(output.len(), 3);
    assert!(output.ends_with_line_end());
    let control = output.start_control();
    output.push_text("X");
    output.finish_control(control);
    assert_eq!(output.since(&element), "\n\nX");
    assert_eq!(output.len(), 4);
    output.commit(element);
    assert_eq!(output.control_ends.as_slice(), [4]);
    assert_places_are_true(&output, "after the commit");
    output.push_text("b");
    assert_eq!(output.into_text(), "a\n\nX b");
}

#[test]
fn a_space_written_before_a_control_moves_the_hidden_spaces_after_it() {
    // ~keep No path of the writer finishes a control while a checkpoint in it is open.
    // ~keep `insert_at` keeps the hole true all the same: it is at 4, then at 5.
    let mut output = PlainOutput::default();
    output.push_text("aaa");
    let control = output.start_control();
    output.push_text("W  ");
    let element = output.open();
    output.end_block();
    output.push_text("zz");
    output.finish_control(control);
    assert_eq!(output.text.whole(), "aaa W  \n\nzz");
    assert_eq!(output.len(), 9);
    assert_eq!(output.since(&element), "\n\nzz");
    output.commit(element);
    assert_eq!(output.control_ends.as_slice(), [9]);
    assert_eq!(output.into_text(), "aaa W\n\nzz");
}

#[test]
fn a_commit_moves_the_start_of_an_open_control_after_the_hidden_spaces() {
    // ~keep No path of the writer commits a checkpoint while a control in it is open. `cut`
    // ~keep keeps the start true all the same: the control starts at 5, then at 3.
    let mut output = PlainOutput::default();
    output.push_text("a  ");
    let element = output.open();
    output.end_block();
    let control = output.start_control();
    output.push_text("X");
    output.commit(element);
    assert_places_are_true(&output, "after the commit");
    output.finish_control(control);
    assert_eq!(output.control_ends.as_slice(), [4]);
    assert_eq!(output.into_text(), "a\n\nX");
}

#[test]
fn the_text_since_a_checkpoint_in_hidden_spaces_starts_after_them() {
    let mut output = PlainOutput::default();
    output.push_text("a ");
    let element = output.open();
    output.push_text(" ");
    output.end_block();
    assert_eq!(output.since(&element), "\n\n");
    output.rollback(element);
    assert_eq!(output.into_text(), "a ");
}
