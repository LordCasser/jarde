import java.util.*;
public class SE {
    static String sideEffect(Optional<String> o){
        StringBuilder sb = new StringBuilder();
        o.ifPresent(sb::append);
        return sb.toString();
    }
    public static void main(String[] a){ System.out.println("["+sideEffect(Optional.of("S"))+"]/["+sideEffect(Optional.empty())+"]"); }
}
