from requests import request


class SpoolerApp:
    value: int

    def __init__(self) -> None:
        self.value = 0
        print("initialized class")
        print(self.__dict__)

    def testFn(self) -> None:
        request.get("https:://google.com", method="GET")
