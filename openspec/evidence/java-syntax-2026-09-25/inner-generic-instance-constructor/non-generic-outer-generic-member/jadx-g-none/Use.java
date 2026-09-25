package minimal;

import java.util.Objects;
import minimal.Outer.Inner;

/* JADX INFO: loaded from: Use.class */
public final class Use {
    public static Outer.Inner<Integer> make(Outer outer, int i) {
        Objects.requireNonNull(outer);
        return outer.new Inner<>(Integer.valueOf(i));
    }

    public static void main(String[] strArr) {
        System.out.println(make(new Outer(), 7).value());
    }
}
