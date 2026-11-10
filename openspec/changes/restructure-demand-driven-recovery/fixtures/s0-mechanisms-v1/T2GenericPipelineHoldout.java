public class T2GenericPipelineHoldout {
    static String trace = "";
    interface Source<T> { T read(int n); }
    interface Use<T> { String show(T item); }
    static class Cell<T> { T item; Cell(T x) { item = x; } <R> Cell<R> map(java.util.function.Function<T,R> f) { return new Cell<R>(f.apply(item)); } }
    static String label(Integer x) { trace += "L"; return "i" + x; }
    public static String run(int n) {
        trace = "";
        Source<Integer> source = k -> { trace += "r"; return k * 2; };
        Cell<String> cell = new Cell<Integer>(source.read(n)).map(T2GenericPipelineHoldout::label);
        Use<String> use = x -> { trace += "u"; return x + ":" + x.length(); };
        return use.show(cell.item) + ":" + trace;
    }
}
