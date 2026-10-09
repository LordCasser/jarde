public class SameNameOverload<T> { public String selected; public void pick(T x) { selected="generic"; } public void pick(String x) { selected="string"; } public void relay(T x) { pick(x); } }
