from miso import Codec

def test_ping():
    c = Codec()
    assert c.ping() == 'pong'

