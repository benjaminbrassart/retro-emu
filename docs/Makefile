SRC = architecture.mmd components.mmd
IMAGES = $(SRC:.mmd=.png)

MMDC = mmdc
MMDCFLAGS =

all: $(IMAGES)

clean:
	$(RM) $(IMAGES)

%.png: %.mmd
	$(MMDC) -i $< -o $@ $(MMDCFLAGS)
