import java.util.Collections;
public class SCGA<T extends Comparable<T>> {
    public T v;
    public SCGA() {}
    public void put(T x) { Collections.singletonList(x); this.v=x; }
    public static void main(String[] args) { SCGA<String> z=new SCGA<String>(); z.put("same-class"); System.out.println("same-class:"+z.v); }
}
