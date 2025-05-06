import ErgoLibC
import Foundation

enum NodeConfError: Error {
    case InvalidScheme
    case MissingHost
    case MissingPort
}

class NodeConf {
    internal var pointer: NodeConfPtr

    internal init(withRawPointer ptr: NodeConfPtr) {
        self.pointer = ptr
    }

    init(withAddrString addrStr: String) throws {
        var ptr: NodeConfPtr?
        let error = addrStr.withCString { cs in
            ergo_lib_node_conf_from_addr(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }

    convenience init(withUrl url: URL) throws {
        guard url.scheme == "http" || url.scheme == "https" else {
            throw NodeConfError.InvalidScheme
        }
        guard url.host != nil else { throw NodeConfError.MissingHost }
        guard url.port != nil else { throw NodeConfError.MissingPort }
        let addr = url.host()! + ":" + String(url.port!)
        try self.init(withAddrString: addr)
    }

    deinit {
        ergo_lib_node_conf_delete(self.pointer)
    }
}
