import java.util.*;
public class IT {
    static class IntRange implements Iterator<Integer>, Iterable<Integer> {   // 手写 Iterator（状态字段跨方法）
        private int cur; private final int end;
        IntRange(int s, int e){ cur = s; end = e; }
        public boolean hasNext(){ return cur < end; }
        public Integer next(){ return cur++; }
        public void remove(){ throw new UnsupportedOperationException(); }
        public Iterator<Integer> iterator(){ return this; }
    }
    static int sumRange(int s, int e){                                        // 消费手写迭代器
        int t = 0;
        for(int v : new IntRange(s, e)){ t += v; }
        return t;
    }
    static String biConsume(Map<String,Integer> m){                           // JDK BiConsumer 位 lambda
        StringBuilder sb = new StringBuilder();
        m.forEach((k, v) -> sb.append(k).append('=').append(v).append(';'));
        return sb.toString();
    }
    public static void main(String[] a){ Map<String,Integer> m = new LinkedHashMap<>(); m.put("a",1); m.put("b",2);
        System.out.println(""+sumRange(1,5)+"/"+biConsume(m)+"/"+new IntRange(2,4).hasNext()); }
}
