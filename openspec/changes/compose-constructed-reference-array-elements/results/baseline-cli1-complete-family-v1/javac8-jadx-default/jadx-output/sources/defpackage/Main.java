package defpackage;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.HashSet;

/* JADX INFO: loaded from: javac8.jar:Main.class */
public class Main {
    public static void event(String str, String str2) {
        System.out.print(str);
        System.out.print(':');
        System.out.println(str2);
    }

    public static String mark(String str) {
        event("mark", str);
        return str;
    }

    public static CharSequence[] sequence() {
        return new CharSequence[]{new StringBuilder(mark("sequence-first")), new StringBuffer(mark("sequence-second"))};
    }

    public static Collection<?>[] collections() {
        return new Collection[]{new ArrayList(Arrays.asList(mark("collection-first"))), new HashSet(Arrays.asList(mark("collection-second")))};
    }

    public static Throwable[] failures() {
        return new Throwable[]{new IllegalStateException(mark("throwable-first")), new IllegalArgumentException(mark("throwable-second"))};
    }

    public static Base[] ownDirect() {
        return new Base[]{new DirectA(mark("direct-first")), new DirectB(mark("direct-second"))};
    }

    public static Base[] ownTwoHop() {
        return new Base[]{new TwoHop(mark("two-hop-first")), new DirectB(mark("two-hop-second"))};
    }

    public static LocalInterface[] ownInterface() {
        return new LocalInterface[]{new DirectA(mark("interface-first")), new TwoHop(mark("interface-second"))};
    }

    private static void observe(Object[] objArr) {
        Object obj = objArr[0];
        if (obj == null) {
            System.out.println("null");
        } else {
            System.out.println(obj.getClass().getName());
        }
        Object obj2 = objArr[1];
        if (obj2 == null) {
            System.out.println("null");
        } else {
            System.out.println(obj2.getClass().getName());
        }
    }

    public static void main(String[] strArr) {
        observe(sequence());
        observe(collections());
        observe(failures());
        observe(ownDirect());
        observe(ownTwoHop());
        observe(ownInterface());
    }
}
