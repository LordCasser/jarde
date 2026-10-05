import java.util.*;
public class IS {
    static class User { final int age; User(int a){ age = a; } public String toString(){ return ""+age; } }
    static List byAnon(List in){
        List c = new ArrayList(in);
        Collections.sort(c, new Comparator(){
            public int compare(Object a, Object b){ return ((User) a).age - ((User) b).age; } });
        return c;
    }
    public static void main(String[] a){ List l = new ArrayList(); l.add(new User(30)); l.add(new User(10)); l.add(new User(20));
        System.out.println(byAnon(l)); }
}
