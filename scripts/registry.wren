class Registry {
    static nodeTypes { __nodeTypes }
    foreign static addNode(klass) {
        __nodeTypes.add(klass)
    }
}

