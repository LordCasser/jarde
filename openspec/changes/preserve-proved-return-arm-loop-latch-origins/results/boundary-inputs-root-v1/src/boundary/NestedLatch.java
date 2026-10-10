package boundary; public final class NestedLatch { public static int nested(int n) { while(n>0) { if(n==1) return n; while(n>1) n--; n--; } return 0; } }
