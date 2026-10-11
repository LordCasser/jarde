# Private candidate review

The candidate changes only `the_ascii_presentations_are_byte_for_byte_the_pre_change_ones` in `tests/recover_unicode_identifiers.rs`, renaming it to `the_ascii_presentations_only_change_in_the_two_proved_f1_for_headers`.

For label `F1`, expected text starts as the historical golden and applies exactly two explicit block substitutions: the `stepTwo` length/+2 loop and the `stepTwoWithCall` length/+2 loop become `for` headers with the existing increment in the sole update clause. Each old block must occur exactly once before replacement. The existing whole-text equality then keeps every remaining F1 byte fixed. `SB`, `RG`, and `package-info` continue comparing against their full frozen text unchanged, and the later package-info boundary assertion is unchanged.

The historical `F1.jarde.java` file is only read and pinned; it is not edited. The workspace-v3 raw is included as failure context; its output shows the old golden assertion failing on the two now-proved F1 header projections. This candidate has not been compiled or run. No shared file was written, and no fixture was added, so this prep does not expand the source pin closure.
