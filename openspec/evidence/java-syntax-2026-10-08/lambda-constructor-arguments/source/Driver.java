public class Driver {
    public static void main(String[] args) throws Exception {
        FunctionalConstructors.trace=0;
        FunctionalConstructors.runnable().run();
        FunctionalConstructors.captured(4).run();
        FunctionalConstructors.overload().run();
        System.out.println("run="+FunctionalConstructors.trace);
        FunctionalConstructors.trace=0;
        Thread t=FunctionalConstructors.ordered(3);
        System.out.println("created="+FunctionalConstructors.trace+":"+t.getName());
        t.run(); System.out.println("invoked="+FunctionalConstructors.trace);
        java.util.PriorityQueue<Integer> q=FunctionalConstructors.comparator();
        q.add(3);q.add(1);q.add(2); System.out.println("queue="+q.poll()+q.poll()+q.poll());
        q=FunctionalConstructors.reference();q.add(3);q.add(1);q.add(2);System.out.println("ref="+q.poll()+q.poll()+q.poll());
        java.util.concurrent.FutureTask<Integer> f=FunctionalConstructors.callable();f.run();System.out.println("callable="+f.get());
        System.out.println("primitive="+FunctionalConstructors.primitive(5).op.applyAsInt(3));
        System.out.println("primitiveRef="+FunctionalConstructors.primitiveReference().op.applyAsInt(3));
    }
}
