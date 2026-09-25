// P3 2c.10/2c.18: the two field-increment shapes javac emits for one instance field.
//
//     javac --release 8 -g:none -d v8 Bump.java
//
// `post` leaves the field's **old** value for the `ireturn` (`n++`), `pre` leaves the sum
// (`++n`). The two differ only in where the `dup_x1` sits, and neither is a `this.n =` in the
// source the recovery writes.
final class Bump {
    int n;

    int post() {
        return this.n++;
    }

    int pre() {
        return ++this.n;
    }
}
