class Game {
    foreign static getChildren(node)
    foreign static getParent(node)
}

class Node {
    children { Registry.getChildren(this) }
    parent { Registry.getParent(this) }
    childrenOfType(klass) { Registry.getChildrenOfType(klass) }
}

class TestNode is Sprite {
    construct new() {
        _pos = this.childrenOfType(Position).pos
        _sprite = this.children.
    }
    update() {
        
    }
}
Registry.addNode(TestNode)
