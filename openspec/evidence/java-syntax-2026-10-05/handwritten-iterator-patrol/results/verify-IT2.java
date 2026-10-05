import java.util.*;
public class IT2 {
    static class IntRange implements Iterator<Integer>, Iterable<Integer> {
        private int cur; private final int end;
        IntRange(int s, int e){ cur = s; end = e; }
        public boolean hasNext(){ return cur < end; }
        public Integer next(){ return cur++; }
        public void remove(){ throw new UnsupportedOperationException(); }
        public Iterator<Integer> iterator(){ return this; }
    }
    static int sumRange(int arg0, int arg1){
        int local2; java.util.Iterator local3;
        local2 = 0;
        local3 = new IntRange(arg0, arg1).iterator();
        while (local3.hasNext()) { int local4 = ((java.lang.Integer) local3.next()).intValue(); local2 += local4; }
        return local2;
    }
    static String biConsume(Map arg0){
        StringBuilder local1 = new StringBuilder();
        arg0.forEach((java.util.function.BiConsumer) ((java.lang.Object p0, java.lang.Object p1) -> local1.append((String) p0).append('=').append((Integer) p1).append(';')));
        return local1.toString();
    }
    public static void main(String[] a){ Map<String,Integer> m = new LinkedHashMap<>(); m.put("a",1); m.put("b",2);
        System.out.println(""+sumRange(1,5)+"/"+biConsume(m)); }
}
