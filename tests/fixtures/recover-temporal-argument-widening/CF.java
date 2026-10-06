import java.util.concurrent.*;
public class CF {
    static String chain() throws Exception {                        // 同步链
        CompletableFuture<String> f = CompletableFuture.supplyAsync(() -> "data");
        String r = f.thenApply(s -> s + "!").thenApply(String::toUpperCase).get();
        return r;
    }
    static String recover() throws Exception {                      // exceptionally 恢复链
        CompletableFuture<String> boom = CompletableFuture.supplyAsync(() -> { throw new IllegalStateException("x"); });
        return boom.exceptionally(e -> "fallback:" + e.getMessage()).get();
    }
    static String combined() throws Exception {                     // thenCombine 两链合流
        CompletableFuture<Integer> a = CompletableFuture.completedFuture(2);
        CompletableFuture<Integer> b = CompletableFuture.supplyAsync(() -> 3);
        return a.thenCombine(b, (x, y) -> x * y).get().toString();
    }
    public static void main(String[] x) throws Exception {
        System.out.println(chain());
        System.out.println(recover());
        System.out.println(combined());
    }
}
