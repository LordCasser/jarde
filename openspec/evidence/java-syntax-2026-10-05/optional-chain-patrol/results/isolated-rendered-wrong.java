import java.util.*;
public class SE2 {                                       // jarde 渲染的 sideEffect（ifPresent 吞掉）
    static String sideEffect(Optional<String> arg0){
        StringBuilder local1 = new StringBuilder();
        return local1.toString();
    }
    public static void main(String[] a){ System.out.println("["+sideEffect(Optional.of("S"))+"]/["+sideEffect(Optional.empty())+"]"); }
}
