import java.util.*;
public class OP {
    static String name(Map<String,String> m, String k){                    // ofNullable+map+orElse 链
        return Optional.ofNullable(m.get(k)).map(String::trim).filter(s -> !s.isEmpty()).orElse("none");
    }
    static int len(Optional<String> o){                                     // isPresent+get 老 API 形
        if(o.isPresent()){ return o.get().length(); }
        return 0;
    }
    static Optional<Integer> parse(String s){                               // 工厂方法三分支
        try{ return Optional.of(Integer.parseInt(s)); }
        catch(NumberFormatException e){ return Optional.empty(); }
    }
    static String sideEffect(Optional<String> o){                           // ifPresent+方法引用副作用
        StringBuilder sb = new StringBuilder();
        o.ifPresent(sb::append);
        return sb.toString();
    }
    public static void main(String[] a){ Map<String,String> m = new HashMap<>(); m.put("x", " hi "); m.put("e", "  ");
        System.out.println(""+name(m,"x")+"/"+name(m,"e")+"/"+name(m,"z")+"/"+len(Optional.of("abc"))+"/"+len(Optional.empty())+"/"+parse("42").get()+"/"+parse("no").isPresent()+"/"+sideEffect(Optional.of("S"))+"/"+sideEffect(Optional.empty())); }
}
