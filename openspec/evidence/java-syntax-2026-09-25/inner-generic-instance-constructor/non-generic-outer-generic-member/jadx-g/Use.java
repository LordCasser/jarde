package minimal;

import java.util.Objects;
import minimal.Outer.Inner;

/* JADX INFO: loaded from: Use.class */
public final class Use {
    public static Outer.Inner<Integer> make(Outer outer, int value) {
        Objects.requireNonNull(outer);
        return outer.new Inner<>(Integer.valueOf(value));
    }

    public static void main(String[] args) {
        System.out.println(make(new Outer(), 7).value());
    }
}
