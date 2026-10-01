import java.util.AbstractList;
import java.util.ArrayList;
import java.util.Date;
import java.util.EnumSet;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

/// The collection-widening negative family: every call below is a real Java 8 widening the closed
/// direct-edge table deliberately does not state, so the recovered source must keep its refusal.
/// Each is a different boundary — a user class that implements `List` only through `AbstractList`,
/// a user subclass of a table class, a `java.util` subpackage, two `java.util` collection classes
/// left out of the enumerated domain, and a `java.util` type outside the collection hierarchy. The
/// original class still runs, because the bytecode it was compiled from is valid Java 8.
public class CWN {
    static class MyList extends AbstractList<String> {
        public String get(int index) {
            return "user";
        }

        public int size() {
            return 1;
        }
    }

    static class MySubList extends ArrayList<String> {
        private static final long serialVersionUID = 1L;
    }

    enum Kind {
        ONE
    }

    static String take(List<String> l) {
        return l.get(0);
    }

    static String mapValue(Map<String, String> m) {
        return m.get("k");
    }

    static int setSize(Set<Kind> s) {
        return s.size();
    }

    static String describe(Comparable<?> c) {
        return c.toString();
    }

    public static void main(String[] args) {
        System.out.println("myList:" + take(new MyList()));

        MySubList subList = new MySubList();
        subList.add("sub");
        System.out.println("subList:" + take(subList));

        ConcurrentHashMap<String, String> concurrent = new ConcurrentHashMap<String, String>();
        concurrent.put("k", "concurrent");
        System.out.println("concurrent:" + mapValue(concurrent));

        EnumSet<Kind> enumSet = EnumSet.of(Kind.ONE);
        System.out.println("enumSet:" + setSize(enumSet));

        IdentityHashMap<String, String> identity = new IdentityHashMap<String, String>();
        identity.put("k", "identity");
        System.out.println("identity:" + mapValue(identity));

        System.out.println("date:" + describe(new Date(0L)).length());
    }
}