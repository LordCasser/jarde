public class CF extends java.lang.Object {
    static java.lang.String chain() throws java.lang.Exception {
        java.util.concurrent.CompletableFuture local0 = java.util.concurrent.CompletableFuture.supplyAsync((java.util.function.Supplier) (() -> "data"));
        java.lang.String local1 = (java.lang.String) local0.thenApply((java.util.function.Function) ((java.lang.Object arg0) -> (java.lang.String) arg0 + "!")).thenApply((java.util.function.Function) ((java.lang.Object p0_) -> ((java.lang.String) p0_).toUpperCase())).get();
        return local1;
    }

    static java.lang.String recover() throws java.lang.Exception {
        java.util.concurrent.CompletableFuture local0 = java.util.concurrent.CompletableFuture.supplyAsync((java.util.function.Supplier) (() -> { throw new java.lang.IllegalStateException("x"); }));
        return (java.lang.String) local0.exceptionally((java.util.function.Function) ((java.lang.Object arg0) -> "fallback:" + ((java.lang.Throwable) arg0).getMessage())).get();
    }
    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        System.out.println(chain());
        System.out.println(recover());
        return;
    }
}
