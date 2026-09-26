package probe;

import java.util.Arrays;

/* JADX INFO: loaded from: enum-arity.jar:probe/EnumArityRunner.class */
public final class EnumArityRunner {
    public static void main(String[] strArr) {
        System.out.println("empty=" + Empty.values().length);
        System.out.println("one=" + One.ONLY.name() + ":" + One.ONLY.ordinal() + "/" + One.values().length);
        System.out.println("four=" + Arrays.toString(Four.values()));
    }
}
