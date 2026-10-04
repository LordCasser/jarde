import static java.util.Arrays.asList;
import static java.lang.Math.max;
public class KX {
    static int dw(int n){ int i=0, s=0; do { s+=i; i++; } while(i<n); return s; }   // do-while
    static int mixed(boolean c){ return c ? max(1,2) : 0; }                          // 三元+静态导入+装箱 int
    static String va(String f, Object... xs){ return String.format(f, xs); }         // 可变参数 Object...
    static int nestedTry(int n){
        try { if(n<0) throw new IllegalArgumentException("neg"); return 100/n; }
        catch(IllegalArgumentException e){ try { return nestedTry(-n); } catch(Exception x){ return -1; } } // catch 内嵌 try
        finally { System.out.println("ft:"+n); }
    }
    public static void main(String[] a){
        System.out.println(""+dw(5)+"/"+mixed(true)+"/"+va("%s-%d","x",7)+"/"+nestedTry(4)+"/"+nestedTry(0));
    }
}
