__all__ = ['JsonException', 'SigmaSerializationException', 'SigmaParsingException', 'WalletException', 'RegisterValueException']
class JsonException(Exception):
    """Error during JSON (de)serialization"""
    ...
class RegisterValueException(Exception):
    """Error fetching register (register was not parsed properly and can't be converted to :py:class:`Constant`)"""
    ...
class SigmaSerializationException(Exception):
    """Error serializing type to bytes"""
    ...

class SigmaParsingException(Exception):
    """Error parsing type from bytes"""
    ...
class WalletException(Exception):
    """Error during signing or wallet-related operation"""
