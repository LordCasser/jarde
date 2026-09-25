class MemberPlacementTarget {
    @MemberPlacement String field;

    @MemberPlacement String method(@MemberPlacement String parameter) {
        return (@MemberPlacement String) parameter;
    }
}
