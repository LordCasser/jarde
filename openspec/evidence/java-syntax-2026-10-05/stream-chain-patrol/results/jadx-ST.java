package defpackage;

/* JADX INFO: loaded from: ST.class */
public class ST {
    static java.util.List<java.lang.String> names() {
        return java.util.Arrays.asList("alice", "Bob", "carol", "dave");
    }

    static java.util.List<java.lang.String> uppers() {
        return (java.util.List) names().stream().map((v0) -> {
            return v0.toUpperCase();
        }).collect(java.util.stream.Collectors.toList());
    }

    static java.util.List<java.lang.String> filtered() {
        return (java.util.List) names().stream().filter(str -> {
            return str.length() > 3;
        }).sorted().collect(java.util.stream.Collectors.toList());
    }

    static int total() {
        return java.util.Arrays.stream(new int[]{1, 2, 3, 4}).filter(i -> {
            return i % 2 == 0;
        }).sum();
    }

    static java.util.Map<java.lang.Integer, java.util.List<java.lang.String>> grouped() {
        return (java.util.Map) names().stream().collect(java.util.stream.Collectors.groupingBy((v0) -> {
            return v0.length();
        }));
    }

    static java.lang.String joined() {
        return (java.lang.String) names().stream().collect(java.util.stream.Collectors.joining(","));
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + uppers() + "/" + filtered() + "/" + total() + "/" + grouped() + "/" + joined());
    }
}
