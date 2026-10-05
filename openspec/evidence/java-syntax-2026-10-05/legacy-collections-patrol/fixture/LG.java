import java.util.*;
public class LG {
    static int[] pqLambda(int[] xs){                                    // PriorityQueue + lambda 比较器
        PriorityQueue<Integer> q = new PriorityQueue<>((a, b) -> b - a);
        for(int x : xs){ q.offer(x); }
        int[] out = new int[xs.length];
        for(int i = 0; i < out.length; i++){ out[i] = q.poll(); }
        return out;
    }
    static int[] stackOps(int[] xs){                                    // 旧 Stack
        Stack<Integer> s = new Stack<>();
        for(int x : xs){ s.push(x); }
        int first = s.peek();
        int[] out = { first, s.pop(), s.size() };
        return out;
    }
    static java.util.List<Integer> dequeOps(int[] xs){                   // ArrayDeque 双端
        Deque<Integer> d = new ArrayDeque<>();
        for(int x : xs){ d.addFirst(x); }
        d.addLast(99);
        List<Integer> out = new ArrayList<>();
        out.add(d.peekFirst()); out.add(d.peekLast()); out.add(d.size());
        return out;
    }
    @Override protected void finalize() throws Throwable {               // finalize 清理链
        try{ System.out.print("fin;"); } finally{ super.finalize(); }
    }
    public static void main(String[] a){ System.out.println(""+java.util.Arrays.toString(pqLambda(new int[]{3,1,2}))+"/"+java.util.Arrays.toString(stackOps(new int[]{7,8}))+"/"+dequeOps(new int[]{5,6})); Runtime.getRuntime().runFinalization(); }
}
