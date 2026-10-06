public class SC extends java.lang.Object {
    public SC() {
        super();
        return;
    }

    static java.lang.String groupDown() {
        java.util.Map local0 = (java.util.Map) java.util.stream.Stream.of((java.lang.Object[]) new java.lang.String[]{"a", "bb", "cc", "ddd"}).collect((java.util.stream.Collector) java.util.stream.Collectors.groupingBy((java.util.function.Function) ((java.lang.Object p0) -> ((java.lang.String) p0).length()), (java.util.function.Supplier) java.util.TreeMap::new, (java.util.stream.Collector) java.util.stream.Collectors.counting()));
        java.util.Map local1 = (java.util.Map) java.util.stream.Stream.of((java.lang.Object[]) new java.lang.String[]{"a", "bb", "cc"}).collect((java.util.stream.Collector) java.util.stream.Collectors.groupingBy((java.util.function.Function) ((java.lang.Object p0_) -> ((java.lang.String) p0_).length()), (java.util.stream.Collector) java.util.stream.Collectors.mapping((java.util.function.Function) ((java.lang.Object arg0) -> ((java.lang.String) arg0).toUpperCase()), (java.util.stream.Collector) java.util.stream.Collectors.joining((java.lang.CharSequence) ","))));
        return new java.lang.StringBuilder().append((java.lang.Object) local0.get((java.lang.Object) java.lang.Integer.valueOf(2))).append("/").append((java.lang.Object) local0.get((java.lang.Object) java.lang.Integer.valueOf(3))).append("/").append((java.lang.String) local1.get((java.lang.Object) java.lang.Integer.valueOf(2))).toString();
    }

    static java.lang.String partition() {
        java.util.Map local0 = (java.util.Map) java.util.stream.Stream.of((java.lang.Object[]) new java.lang.String[]{"x", "yy", "zzz"}).collect((java.util.stream.Collector) java.util.stream.Collectors.partitioningBy((java.util.function.Predicate) ((java.lang.Object arg0) -> ((java.lang.String) arg0).length() > 1)));
        java.util.Map local1 = (java.util.Map) java.util.stream.Stream.of((java.lang.Object[]) new java.lang.String[]{"k1", "k2"}).collect((java.util.stream.Collector) java.util.stream.Collectors.toMap((java.util.function.Function) ((java.lang.Object p0_) -> ((java.lang.String) p0_).toUpperCase()), (java.util.function.Function) ((java.lang.Object p0__) -> ((java.lang.String) p0__).length()), (java.util.function.BinaryOperator) ((java.lang.Object arg0, java.lang.Object arg1) -> (java.lang.Integer) arg0)));
        return new java.lang.StringBuilder().append(((java.util.List) local0.get((java.lang.Object) java.lang.Boolean.valueOf(true))).size()).append("/").append((java.lang.Object) local1.get((java.lang.Object) "K1")).append("/").append((java.lang.Object) local1.get((java.lang.Object) "K2")).toString();
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println((java.lang.String) groupDown());
        java.lang.System.out.println((java.lang.String) partition());
        return;
    }
}
