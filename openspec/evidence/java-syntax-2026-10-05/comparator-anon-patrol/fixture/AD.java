import java.util.*;
public class AD {
    interface Op { int apply(int x); }
    static <T> int runGen(T o, int v){ return v + 1; }                                     // 泛型 own 方法（单参）
    static int viaGeneric(List<String> l){ return runGen(new Op(){ public int apply(int x){ return x; } }, l.size()); }   // 匿名→泛型 own
    static String viaThread(final String msg){                                              // JDK 非泛型 ctor 实参（Runnable）
        final StringBuilder sb = new StringBuilder();
        Thread t = new Thread(new Runnable(){ public void run(){ sb.append(msg); } });
        t.start();
        try{ t.join(); }catch(InterruptedException e){ }
        return sb.toString();
    }
    static List<String> viaSort(List<String> l){                                            // JDK 泛型方法 + 匿名（CP 复刻）
        List<String> c = new ArrayList<String>(l);
        Collections.sort(c, new Comparator<String>(){ public int compare(String a, String b){ return b.compareTo(a); } });
        return c;
    }
    public static void main(String[] a){ List<String> l = new ArrayList<>(Arrays.asList("a","c","b"));
        System.out.println(""+viaGeneric(new ArrayList<>(l))+"/"+viaThread("T")+"/"+viaSort(l)); }
}
