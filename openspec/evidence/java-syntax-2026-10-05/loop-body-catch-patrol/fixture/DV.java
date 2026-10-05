public class DV {
    static int plainFor(int[] xs){ int g = 0; for(int i = 0; i < xs.length; i++){ try { if(xs[i] < 0){ throw new IllegalArgumentException("n"); } g += xs[i]; } catch(IllegalArgumentException e){ g -= 1; } } return g; }  // 纯 for
    static int whileLoop(int[] xs){ int g = 0; int i = 0; while(i < xs.length){ try { if(xs[i] < 0){ throw new IllegalArgumentException("n"); } g += xs[i]; } catch(IllegalArgumentException e){ g -= 1; } i++; } return g; }  // while
    static int forEach(int[] xs){ int g = 0; for(int x : xs){ try { g += x; } catch(Exception e){ g -= 1; } } return g; }  // for-each 无 throw-if（简化）
    static int loopNoTry(int[] xs){ int g = 0; for(int x : xs){ if(x < 0){ g -= 1; } else { g += x; } } return g; }  // 对照：无 try
    public static void main(String[] a){ System.out.println(""+plainFor(new int[]{1,-2,3})+"/"+whileLoop(new int[]{1,-2,3})+"/"+forEach(new int[]{1,2})+"/"+loopNoTry(new int[]{1,-2,3})); }
}
