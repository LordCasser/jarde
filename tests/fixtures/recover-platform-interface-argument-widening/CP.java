import java.util.*;
public class CP {
    static class User { final String name; final int age; User(String n, int a){ name = n; age = a; } String getName(){ return name; } int getAge(){ return age; }
        public String toString(){ return name + ":" + age; } }
    static List<User> byNameAge(List<User> us){                              // comparing 方法引用链 + reversed
        List<User> c = new ArrayList<User>(us);
        Collections.sort(c, Comparator.comparing(User::getName).thenComparing(User::getAge).reversed());
        return c;
    }
    static List<User> byAnon(List<User> us){                                  // 匿名 Comparator（pre-lambda 形）
        List<User> c = new ArrayList<User>(us);
        Collections.sort(c, new Comparator<User>(){
            public int compare(User a, User b){ return a.age - b.age; } });
        return c;
    }
    static String[] byLen(String[] xs){                                       // 数组 sort + lambda
        String[] c = xs.clone();
        Arrays.sort(c, (a, b) -> a.length() - b.length());
        return c;
    }
    public static void main(String[] a){ List<User> us = new ArrayList<>(); us.add(new User("bo",30)); us.add(new User("al",40)); us.add(new User("al",20));
        System.out.println(""+byNameAge(us)+"/"+byAnon(us)+"/"+java.util.Arrays.toString(byLen(new String[]{"ccc","a","bb"}))); }
}
