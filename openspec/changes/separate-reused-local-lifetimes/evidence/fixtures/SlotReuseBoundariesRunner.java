public class SlotReuseBoundariesRunner {
    public static void main(String[] args) {
        System.out.println(SlotReuseBoundaries.sameType(4));
        System.out.println(SlotReuseBoundaries.exclusiveBranch(true));
        System.out.println(SlotReuseBoundaries.exclusiveBranch(false));
        System.out.println(SlotReuseBoundaries.loopBodyReuse(2));
        System.out.println(SlotReuseBoundaries.handlerReuse(false));
        System.out.println(SlotReuseBoundaries.handlerReuse(true));
        System.out.println(SlotReuseBoundaries.category2Adjacent());
    }
}
