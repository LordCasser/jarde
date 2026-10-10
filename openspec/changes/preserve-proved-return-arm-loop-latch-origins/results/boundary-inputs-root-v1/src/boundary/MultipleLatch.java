package boundary; public final class MultipleLatch { public static int multiple(int n) { while(n>0) { if(n==2) continue; if(n==1) return n; n--; } return 0; } }
