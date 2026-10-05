import java.util.*;
public class AC implements Comparator<String> {              // 顶层具名实现类（无 $）
    public int compare(String a, String b){ return a.length() - b.length(); }
}
