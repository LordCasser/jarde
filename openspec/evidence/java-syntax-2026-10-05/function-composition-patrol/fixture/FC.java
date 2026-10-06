import java.util.function.*;
public class FC {
    static String compose(){                                       // Function 组合
        Function<Integer,Integer> doubleIt = x -> x * 2;
        Function<Integer,Integer> inc = x -> x + 1;
        Function<Integer,Integer> both = doubleIt.compose(inc);    // (x+1)*2
        Function<Integer,Integer> after = doubleIt.andThen(inc);    // x*2+1
        return both.apply(5) + "/" + after.apply(5);
    }
    static String predicates(String s){                            // Predicate 组合
        Predicate<String> nonEmpty = x -> !x.isEmpty();
        Predicate<String> short_ = x -> x.length() < 5;
        Predicate<String> ok = nonEmpty.and(short_).negate();
        return ok.test(s) + "/" + nonEmpty.or(short_).test("");
    }
    static String bi(){                                            // BiFunction/供应商
        BiFunction<Integer,Integer,Integer> add = (a, b) -> a + b;
        Supplier<String> mk = () -> "v" + add.apply(2, 3);
        return mk.get();
    }
    public static void main(String[] x){
        System.out.println(compose());
        System.out.println(predicates("hello"));
        System.out.println(bi());
    }
}
