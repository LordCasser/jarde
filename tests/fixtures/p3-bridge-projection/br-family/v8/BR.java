import java.util.*;
public class BR {
    interface Node { Node next(); }
    static class Base implements Node {
        public Base next() { return new Base(); }   // covariant return -> bridge
    }
    static class Box<T> {
        T get() { return null; }
        void set(T v) { }
    }
    static class StrBox extends Box<String> {
        @Override String get() { return "s"; }        // bridge for get
        @Override void set(String v) { }              // bridge for set
    }
    static class Impl implements Comparable<Impl> {
        public int compareTo(Impl o) { return 0; }    // bridge compareTo(Object)
    }
    public static void main(String[] a) {
        Node n = new Base();
        System.out.println(n.next().getClass().getSimpleName());
        StrBox sb = new StrBox();
        System.out.println(sb.get());
        System.out.println(new Impl().compareTo(new Impl()));
    }
}
