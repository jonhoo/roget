#!/usr/bin/Rscript

data <- read.table('entropy.dat', col.names = c("entropy", "guesses"))

library(ggplot2)

reg <- lm(exp(guesses) ~ entropy, data = data)

p <- ggplot(data, aes(entropy, guesses)) + geom_point()
xs <- seq(0,8,length=36)
pred <- predict(reg, newdata = data.frame(entropy = xs))
pred <- log(pred)
pred <- data.frame(entropy = xs, guesses = pred)
also <- log(xs * 3.996 + 4.121)
also <- data.frame(entropy = xs, guesses = also)
print(pred)
p <- p + geom_point(data = pred, aes(entropy, guesses), color = "red")
p <- p + geom_point(data = also, aes(entropy, guesses), color = "blue")
ggsave("plot.png", p)
print(reg)
