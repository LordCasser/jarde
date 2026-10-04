public class SY {
    static int mon(int n){ synchronized(SY.class){ return n*2; } }                        // 类锁 + return
    static int monObj(Object l, int n){ synchronized(l){ if(n>0) return 1; return 0; } }   // 对象锁 + 早退
    static int monNested(int n){ synchronized(SY.class){ synchronized(SY.class){ return n; } } } // 嵌套 monitor（同锁）
    static int doCont(int n){ int i=0,s=0; do { i++; if(i%2==0) continue; s+=i; } while(i<n); return s; } // do-while+continue
    static int labMix(int n){ int s=0; outer: for(int a=0;a<n;a++){ for(int b=0;b<n;b++){ if(b>a) continue outer; if(a==3) break outer; s+=b; } } return s; } // 混合标签
    public static void main(String[] a){ Object o=new Object(); System.out.println(""+mon(3)+"/"+monObj(o,1)+"/"+monNested(7)+"/"+doCont(6)+"/"+labMix(5)); }
}
