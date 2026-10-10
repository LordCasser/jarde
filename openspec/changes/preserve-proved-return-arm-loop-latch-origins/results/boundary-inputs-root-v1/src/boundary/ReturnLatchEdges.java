package boundary; public final class ReturnLatchEdges { public static int edge(int n) { try { while(n>0) { if(n==1) return n; n--; } } catch(RuntimeException ignored) { return -1; } return 0; } }
