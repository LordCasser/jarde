import java.util.*;
import java.util.stream.*;
public class SC {
    static String groupDown(){                                     // 三参 groupingBy + 下游 mapping/counting
        Map<Integer, Long> counts = Stream.of("a","bb","cc","ddd")
            .collect(Collectors.groupingBy(String::length, TreeMap::new, Collectors.counting()));
        Map<Integer, String> joined = Stream.of("a","bb","cc")
            .collect(Collectors.groupingBy(String::length, Collectors.mapping(s -> s.toUpperCase(), Collectors.joining(","))));
        return counts.get(2) + "/" + counts.get(3) + "/" + joined.get(2);
    }
    static String partition(){                                     // partitioningBy + toMap 合并
        Map<Boolean, List<String>> p = Stream.of("x","yy","zzz").collect(Collectors.partitioningBy(s -> s.length() > 1));
        Map<String, Integer> m = Stream.of("k1","k2").collect(Collectors.toMap(String::toUpperCase, String::length, (a, b) -> a));
        return p.get(true).size() + "/" + m.get("K1") + "/" + m.get("K2");
    }
    public static void main(String[] a){ System.out.println(groupDown()); System.out.println(partition()); }
}
