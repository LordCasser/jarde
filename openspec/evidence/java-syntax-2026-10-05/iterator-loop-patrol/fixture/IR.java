import java.util.*;
public class IR {
    static List<String> dropShort(List<String> in){                       // 手动 Iterator + remove（经典）
        Iterator<String> it = in.iterator();
        while(it.hasNext()){ String s = it.next(); if(s.length() < 3){ it.remove(); } }
        return in;
    }
    static List<String> replaceAll(List<String> in){                        // ListIterator.set
        ListIterator<String> li = in.listIterator();
        while(li.hasNext()){ String s = li.next(); if(s.equals("x")){ li.set("EX"); } }
        return in;
    }
    static List<String> modern(List<String> in){ in.removeIf(s -> s.isEmpty()); return in; }   // removeIf lambda
    public static void main(String[] a){ System.out.println(""+dropShort(new ArrayList<>(Arrays.asList("a","bb","ccc")))+"/"+replaceAll(new ArrayList<>(Arrays.asList("x","y")))+"/"+modern(new ArrayList<>(Arrays.asList("","z")))); }
}
