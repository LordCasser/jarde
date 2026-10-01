import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Deque;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Hashtable;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;
import java.util.NavigableMap;
import java.util.NavigableSet;
import java.util.Properties;
import java.util.Queue;
import java.util.Set;
import java.util.SortedMap;
import java.util.SortedSet;
import java.util.Stack;
import java.util.TreeMap;
import java.util.TreeSet;
import java.util.Vector;

/// The collection-widening positive family: every call below passes a real JDK 8 `java.util`
/// collection value where the callee declares a collection interface, so each argument's presented
/// type is a class (or interface) and the required type is an ancestor interface the closed direct
/// -edge table reaches. `main` prints one labelled line per call.
public class CWV {
    static String mapValue(Map<String, String> m) {
        return m.get("k");
    }

    static int mapSize(Map<?, ?> m) {
        return m.size();
    }

    static int setSize(Set<String> s) {
        return s.size();
    }

    static int collectionSize(Collection<String> c) {
        return c.size();
    }

    static int iterableSize(Iterable<String> it) {
        int n = 0;
        for (String s : it) {
            if (s != null) {
                n++;
            }
        }
        return n;
    }

    static int queueSize(Queue<String> q) {
        return q.size();
    }

    static int sortedSetSize(SortedSet<String> s) {
        return s.size();
    }

    static String sortedMapValue(SortedMap<String, String> m) {
        return m.get("k");
    }

    static String listValue(List<String> l) {
        return l.get(0);
    }

    static int nestedListSize(List<List<String>> ls) {
        return ls.size() + ls.get(0).size();
    }

    static int dequeSize(Deque<String> d) {
        return d.size();
    }

    static int dequeToQueue(Deque<String> d) {
        return queueSize(d);
    }

    static int sortedSetToSet(SortedSet<String> s) {
        return setSize(s);
    }

    static int navigableSetToSortedSet(NavigableSet<String> s) {
        return sortedSetSize(s);
    }

    static String navigableMapToSortedMap(NavigableMap<String, String> m) {
        return sortedMapValue(m);
    }

    static int listToCollection(List<String> l) {
        return collectionSize(l);
    }

    static int setToCollection(Set<String> s) {
        return collectionSize(s);
    }

    static int queueToCollection(Queue<String> q) {
        return collectionSize(q);
    }

    static int collectionToIterable(Collection<String> c) {
        return iterableSize(c);
    }

    public static void main(String[] args) {
        HashMap<String, String> hashMap = new HashMap<String, String>();
        hashMap.put("k", "hashmap");
        System.out.println("map:" + mapValue(hashMap));

        TreeMap<String, String> treeMap = new TreeMap<String, String>();
        treeMap.put("k", "treemap");
        System.out.println("treemap:" + mapValue(treeMap));

        LinkedHashMap<String, String> linkedMap = new LinkedHashMap<String, String>();
        linkedMap.put("k", "linkedmap");
        System.out.println("linkedmap:" + mapValue(linkedMap));

        Hashtable<String, String> hashtable = new Hashtable<String, String>();
        hashtable.put("k", "hashtable");
        System.out.println("hashtable:" + mapValue(hashtable));

        Properties properties = new Properties();
        properties.setProperty("k", "properties");
        System.out.println("properties:" + mapSize(properties));

        HashSet<String> hashSet = new HashSet<String>();
        hashSet.add("hashset");
        System.out.println("set:" + setSize(hashSet));

        TreeSet<String> treeSet = new TreeSet<String>();
        treeSet.add("treeset");
        System.out.println("treeset:" + setSize(treeSet));

        LinkedHashSet<String> linkedHashSet = new LinkedHashSet<String>();
        linkedHashSet.add("linkedhashset");
        System.out.println("linkedhashset:" + setSize(linkedHashSet));

        ArrayList<String> arrayList = new ArrayList<String>();
        arrayList.add("coll");
        arrayList.add("two");
        System.out.println("collection:" + collectionSize(arrayList));
        System.out.println("list:" + listValue(arrayList));
        System.out.println("iterable:" + iterableSize(arrayList));

        LinkedList<String> linkedList = new LinkedList<String>();
        linkedList.add("linked");
        System.out.println("linkedlist:" + listValue(linkedList));

        Vector<String> vector = new Vector<String>();
        vector.add("vector");
        System.out.println("vector:" + collectionSize(vector));

        Stack<String> stack = new Stack<String>();
        stack.push("stack");
        System.out.println("stack:" + listValue(stack));

        ArrayList<ArrayList<String>> elementList = new ArrayList<ArrayList<String>>();
        ArrayList<String> inner = new ArrayList<String>();
        inner.add("inner");
        elementList.add(inner);
        System.out.println("nestedList:" + listValue(elementList.get(0)));

        ArrayList<List<String>> nestedList = new ArrayList<List<String>>();
        nestedList.add(inner);
        System.out.println("nested:" + nestedListSize(nestedList));

        ArrayDeque<String> arrayDeque = new ArrayDeque<String>();
        arrayDeque.add("deque");
        System.out.println("deque:" + dequeSize(arrayDeque));
        System.out.println("dequeCollection:" + collectionSize(arrayDeque));

        List<String> listVar = new ArrayList<String>();
        listVar.add("listinterface");
        listVar.add("second");
        System.out.println("listToCollection:" + listToCollection(listVar));

        Set<String> setVar = new HashSet<String>();
        setVar.add("setinterface");
        System.out.println("setToCollection:" + setToCollection(setVar));

        Queue<String> queueVar = new LinkedList<String>();
        queueVar.add("queueinterface");
        System.out.println("queueToCollection:" + queueToCollection(queueVar));

        Collection<String> collectionVar = new ArrayList<String>();
        collectionVar.add("collectioninterface");
        collectionVar.add("another");
        System.out.println("collectionToIterable:" + collectionToIterable(collectionVar));

        Deque<String> dequeVar = new ArrayDeque<String>();
        dequeVar.add("dequeinterface");
        System.out.println("dequeToQueue:" + dequeToQueue(dequeVar));

        SortedSet<String> sortedSetVar = new TreeSet<String>();
        sortedSetVar.add("sortedsetinterface");
        System.out.println("sortedSetToSet:" + sortedSetToSet(sortedSetVar));

        NavigableSet<String> navigableSetVar = new TreeSet<String>();
        navigableSetVar.add("navigablesetinterface");
        System.out.println("navigableSetToSortedSet:" + navigableSetToSortedSet(navigableSetVar));

        NavigableMap<String, String> navigableMapVar = new TreeMap<String, String>();
        navigableMapVar.put("k", "navigablemapinterface");
        System.out.println("navigableMapToSortedMap:" + navigableMapToSortedMap(navigableMapVar));
    }
}